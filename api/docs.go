package handler

import (
	"encoding/json"
	"net/http"
)

// / OpenAPI structure for the documentation.
type OpenAPI struct {
	Openapi    string             `json:"openapi"`
	Info       OpenAPIInfo        `json:"info"`
	Paths      map[string]APIPath `json:"paths"`
	Servers    []APIServer        `json:"servers"`
	Components OpenAPIComponents  `json:"components"`
}

type OpenAPIInfo struct {
	Title       string `json:"title"`
	Description string `json:"description"`
	Version     string `json:"version"`
}

type APIPath struct {
	Get  *APIEndpoint `json:"get,omitempty"`
	Post *APIEndpoint `json:"post,omitempty"`
}

type APIEndpoint struct {
	Summary     string                 `json:"summary"`
	Description string                 `json:"description"`
	OperationID string                 `json:"operationId"`
	Parameters  []APIParameter         `json:"parameters,omitempty"`
	RequestBody *APIRequestBody        `json:"requestBody,omitempty"`
	Responses   map[string]APIResponse `json:"responses"`
}

type APIParameter struct {
	Name        string    `json:"name"`
	In          string    `json:"in"`
	Required    bool      `json:"required"`
	Schema      APISchema `json:"schema"`
	Description string    `json:"description,omitempty"`
}

type APIRequestBody struct {
	Description string                `json:"description"`
	Content     map[string]APIContent `json:"content"`
	Required    bool                  `json:"required"`
}

type APIContent struct {
	Schema APISchema `json:"schema"`
}

type APISchema struct {
	Type   string `json:"type"`
	Format string `json:"format,omitempty"`
}

type APIResponse struct {
	Description string                `json:"description"`
	Content     map[string]APIContent `json:"content"`
}

type APIServer struct {
	URL         string `json:"url"`
	Description string `json:"description"`
}

type OpenAPIComponents struct {
	Schemas map[string]APISchema `json:"schemas"`
}

// / Handler to serve OpenAPI docs as JSON
func DocsHandler(w http.ResponseWriter, r *http.Request) {
	openAPIDocs := OpenAPI{
		Openapi: "3.0.0",
		Info: OpenAPIInfo{
			Title:       "ifkash.dev API Documentation",
			Description: "API documentation for interacting with ifkash.dev services",
			Version:     "1.0.0",
		},
		Servers: []APIServer{
			{
				URL:         "https://ifkash.dev/api",
				Description: "Production server",
			},
		},
		Paths: map[string]APIPath{
			"/hn": {
				Get: &APIEndpoint{
					Summary:     "Get Top Hacker News Stories",
					Description: "Retrieve top 20 Hacker News stories from the past week.",
					OperationID: "getHNTopStories",
					Responses: map[string]APIResponse{
						"200": {
							Description: "A list of top Hacker News stories",
							Content: map[string]APIContent{
								"application/json": {
									Schema: APISchema{Type: "array", Format: "Story"},
								},
							},
						},
					},
				},
			},
			"/leetcode": {
				Get: &APIEndpoint{
					Summary:     "Get LeetCode Progress",
					Description: "Fetch user progress from LeetCode.",
					OperationID: "getLeetCodeProgress",
					Responses: map[string]APIResponse{
						"200": {
							Description: "User's LeetCode progress",
							Content: map[string]APIContent{
								"application/json": {
									Schema: APISchema{Type: "object", Format: "LeetCodeProgress"},
								},
							},
						},
					},
				},
			},
			"/llm": {
				Post: &APIEndpoint{
					Summary:     "LLM Interaction",
					Description: "Send a prompt to the LLM and get a response.",
					OperationID: "postLLMInteraction",
					RequestBody: &APIRequestBody{
						Description: "Request body containing model and messages",
						Content: map[string]APIContent{
							"application/json": {
								Schema: APISchema{Type: "object"},
							},
						},
						Required: true,
					},
					Responses: map[string]APIResponse{
						"200": {
							Description: "Response from the LLM",
							Content: map[string]APIContent{
								"application/json": {
									Schema: APISchema{Type: "object"},
								},
							},
						},
					},
				},
			},
			"/pwc": {
				Get: &APIEndpoint{
					Summary:     "Papers with Code",
					Description: "Retrieve Papers with Code datasets and models.",
					OperationID: "getPWCData",
					Responses: map[string]APIResponse{
						"200": {
							Description: "List of papers with code",
							Content: map[string]APIContent{
								"application/json": {
									Schema: APISchema{Type: "array", Format: "Paper"},
								},
							},
						},
					},
				},
			},
			"/pwc/paper": {
				Get: &APIEndpoint{
					Summary:     "Papers with Code Paper Lookup",
					Description: "Fetch specific paper details by paper URL.",
					OperationID: "getPWCPaper",
					Parameters: []APIParameter{
						{
							Name:        "paper_url",
							In:          "query",
							Required:    true,
							Schema:      APISchema{Type: "string"},
							Description: "The URL of the paper on Papers with Code",
						},
					},
					Responses: map[string]APIResponse{
						"200": {
							Description: "Paper details",
							Content: map[string]APIContent{
								"application/json": {
									Schema: APISchema{Type: "object", Format: "PaperDetails"},
								},
							},
						},
					},
				},
			},
		},
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(openAPIDocs)
}
