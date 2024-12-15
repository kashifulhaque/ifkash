package handler

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strconv"
	"strings"
)

// Handler is the entry point for the Vercel serverless function.
func Handler(w http.ResponseWriter, r *http.Request) {
	// Parse the request URL path to extract username and count.
	path := strings.TrimPrefix(r.URL.Path, "/api/lc/submissions/")
	parts := strings.Split(path, "/")
	if len(parts) != 2 {
		http.Error(w, "Invalid URL format. Use /api/lc/submissions/USERNAME/COUNT", http.StatusBadRequest)
		return
	}

	username := parts[0]
	count, err := strconv.Atoi(parts[1])
	if err != nil || count < 1 || count > 10 {
		http.Error(w, "COUNT must be an integer between 1 and 10", http.StatusBadRequest)
		return
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
