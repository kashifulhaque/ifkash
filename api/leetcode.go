package handler

import (
	"encoding/json"
	"io/ioutil"
	"net/http"
	"strings"
)

// serveDeez handles the request and returns the transformed JSON response.
func serveDeez(w http.ResponseWriter, r *http.Request) {
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

	res, err := http.DefaultClient.Do(req)
	if err != nil {
		http.Error(w, "Error making request to LeetCode", http.StatusInternalServerError)
		return
	}
	defer res.Body.Close()

	body, err := ioutil.ReadAll(res.Body)
	if err != nil {
		http.Error(w, "Error reading response from LeetCode", http.StatusInternalServerError)
		return
	}

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
	err = json.Unmarshal(body, &response)
	if err != nil {
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
		http.Error(w, "Error marshalling result JSON", http.StatusInternalServerError)
		return
	}

	// Set the response header to application/json
	w.Header().Set("Content-Type", "application/json")
	// Write the JSON response
	w.Write(resultJSON)
}

// Handler is the entry point for the Vercel serverless function
func Handler(w http.ResponseWriter, r *http.Request) {
	serveDeez(w, r)
}
