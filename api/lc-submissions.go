package handler

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
)

func LCSubmissionsHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Only POST method is allowed", http.StatusMethodNotAllowed)
		return
	}

	// Parse the JSON body to extract username and count.
	var reqBody struct {
		Username string `json:"username"`
		Count    int    `json:"count"`
	}
	if err := json.NewDecoder(r.Body).Decode(&reqBody); err != nil {
		http.Error(w, "Invalid request body", http.StatusBadRequest)
		return
	}

	username := reqBody.Username
	count := reqBody.Count
	if username == "" {
		http.Error(w, "Invalid username", http.StatusBadRequest)
		return
	}

	if count < 0 {
		count = 0
	}
	if count > 20 {
		count = 20
	}

	// Prepare the GraphQL payload.
	payload := fmt.Sprintf(`{
		"query": "query recentAcSubmissions($username: String!, $limit: Int!) { recentAcSubmissionList(username: $username, limit: $limit) { id title titleSlug timestamp } }",
		"variables": { "username": "%s", "limit": %d },
		"operationName": "recentAcSubmissions"
	}`, username, count)

	url := "https://leetcode.com/graphql"
	req, err := http.NewRequest("POST", url, strings.NewReader(payload))
	if err != nil {
		http.Error(w, "Failed to create request", http.StatusInternalServerError)
		return
	}
	req.Header.Add("Content-Type", "application/json")

	// Make the HTTP request to LeetCode.
	res, err := http.DefaultClient.Do(req)
	if err != nil {
		http.Error(w, "Error making request to LeetCode", http.StatusInternalServerError)
		return
	}
	defer res.Body.Close()

	// Read and return the response body.
	body, err := io.ReadAll(res.Body)
	if err != nil {
		http.Error(w, "Error reading response from LeetCode", http.StatusInternalServerError)
		return
	}

	// Set response header to JSON and write the response.
	w.Header().Set("Content-Type", "application/json")
	w.Write(body)
}
