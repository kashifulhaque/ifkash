package main



import (
  "encoding/json"

  "fmt"
  "io/ioutil"
  "net/http"
  "strings"
)

func serve_deez(w http.ResponseWriter, r *http.Request) {
  url := "https://leetcode.com/graphql"
  payload := strings.NewReader(`{
    "query": "query userProblemsSolved($username: String!) { allQuestionsCount { difficulty count } matchedUser (username: $username) { submitStatsGlobal { acSubmissionNum { difficulty count } } } }",

    "variables": {
      "username": "ifkash"
    },
    "operationName": "userProblemsSolved"
  }`)

  req, _ := http.NewRequest("POST", url, payload)
  req.Header.Add("Content-Type", "application/json")


  res, _ := http.DefaultClient.Do(req)


  defer res.Body.Close()
  body, _ := ioutil.ReadAll(res.Body)

  // Define a struct to hold the parsed JSON response
  type QuestionCount struct {
    Difficulty string `json:"difficulty"`
    Count      int    `json:"count"`
  }


  type SubmissionNum struct {
    Difficulty string `json:"difficulty"`
    Count      int    `json:"count"`
  }


  type SubmitStatsGlobal struct {
    ACSubmissionNum []SubmissionNum `json:"acSubmissionNum"`
  }


  type MatchedUser struct {

    SubmitStatsGlobal SubmitStatsGlobal `json:"submitStatsGlobal"`

  }

  type Data struct {
    AllQuestionsCount []QuestionCount `json:"allQuestionsCount"`

    MatchedUser       MatchedUser     `json:"matchedUser"`
  }

  type Response struct {
    Data Data `json:"data"`
  }

  var response Response
  err := json.Unmarshal(body, &response)
  if err != nil {
    fmt.Println("Error parsing JSON:", err)

    http.Error(w, "Error parsing JSON", http.StatusInternalServerError)

    return


  }

  // Create a map for the new JSON structure
  result := map[string]map[string]int{
    "easy":   {"complete": 0, "total": 0},
    "medium": {"complete": 0, "total": 0},
    "hard":   {"complete": 0, "total": 0},
    "total":  {"complete": 0, "total": 0},
  }



  // Populate the total counts

  for _, question := range response.Data.AllQuestionsCount {
    switch question.Difficulty {

    case "All":
      result["total"]["total"] = question.Count
    case "Easy":
      result["easy"]["total"] = question.Count

    case "Medium":
      result["medium"]["total"] = question.Count
    case "Hard":

      result["hard"]["total"] = question.Count

    }
  }


  // Populate the complete counts
  for _, submission := range response.Data.MatchedUser.SubmitStatsGlobal.ACSubmissionNum {
    switch submission.Difficulty {
    case "All":
      result["total"]["complete"] = submission.Count
    case "Easy":
      result["easy"]["complete"] = submission.Count
    case "Medium":
      result["medium"]["complete"] = submission.Count
    case "Hard":
      result["hard"]["complete"] = submission.Count
    }
  }


  // Convert the result map to JSON
  resultJSON, err := json.MarshalIndent(result, "", "  ")
  if err != nil {
    fmt.Println("Error marshalling result JSON:", err)
    http.Error(w, "Error marshalling result JSON", http.StatusInternalServerError)

    return
  }

  // Set the response header to application/json
  w.Header().Set("Content-Type", "application/json")
  // Write the JSON response
  w.Write(resultJSON)
}

func Handler() {
  http.HandleFunc("/", serve_deez)
  fmt.Println("Server is listening on port 8080...")
  http.ListenAndServe(":8080", nil)
}

