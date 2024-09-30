package handler

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
)

// RequestBody struct defines the expected structure of the incoming request.
type RequestBody struct {
	Model   string          `json:"model"`
	Messages []Message       `json:"messages"`
	Stream  bool            `json:"stream,omitempty"` // Optional field to enable streaming
}

// Message represents the structure of a single message in the request body.
type Message struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

// ProxyHandler handles the proxy request to Cloudflare Workers AI API.
func ProxyHandler(w http.ResponseWriter, r *http.Request) {
	// Ensure the request is a POST
	if r.Method != http.MethodPost {
		http.Error(w, "Only POST method is allowed", http.StatusMethodNotAllowed)
		return
	}

	// Load environment variables
	cloudflareAccountID := os.Getenv("CLOUDFLARE_ACCOUNT_ID")
	cloudflareAPIKey := os.Getenv("CLOUDFLARE_WORKERS_AI_API_KEY")

	if cloudflareAccountID == "" || cloudflareAPIKey == "" {
		http.Error(w, "Missing required environment variables", http.StatusInternalServerError)
		return
	}

	// Read the body of the incoming request
	reqBody, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, "Failed to read request body", http.StatusBadRequest)
		return
	}
	defer r.Body.Close()

	// Parse the incoming request body to check for the "stream" key
	var requestBody RequestBody
	err = json.Unmarshal(reqBody, &requestBody)
	if err != nil {
		http.Error(w, "Failed to parse request body", http.StatusBadRequest)
		return
	}

	// Create the Cloudflare API URL
	cloudflareAPIURL := fmt.Sprintf("https://api.cloudflare.com/client/v4/accounts/%s/ai/v1/chat/completions", cloudflareAccountID)

	// Create a new request to forward to Cloudflare's API
	cloudflareReq, err := http.NewRequest("POST", cloudflareAPIURL, bytes.NewBuffer(reqBody))
	if err != nil {
		http.Error(w, "Failed to create request to Cloudflare API", http.StatusInternalServerError)
		return
	}

	// Set the required headers for the Cloudflare API
	cloudflareReq.Header.Set("Authorization", "Bearer "+cloudflareAPIKey)
	cloudflareReq.Header.Set("Content-Type", "application/json")

	// Forward the request to Cloudflare's API
	client := &http.Client{}
	resp, err := client.Do(cloudflareReq)
	if err != nil {
		http.Error(w, "Failed to forward request to Cloudflare API", http.StatusBadGateway)
		return
	}
	defer resp.Body.Close()

	// Handle the streaming case
	if requestBody.Stream {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)

		// Stream the response back to the client as chunks arrive
		buf := make([]byte, 1024)
		for {
			n, err := resp.Body.Read(buf)
			if err != nil {
				if err == io.EOF {
					break
				}
				http.Error(w, "Error reading streaming response", http.StatusInternalServerError)
				return
			}
			w.Write(buf[:n])
			w.(http.Flusher).Flush() // Flush the data to the client as it arrives
		}
		return
	}

	// For non-streaming response, simply forward the entire response
	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		http.Error(w, "Failed to read response from Cloudflare API", http.StatusInternalServerError)
		return
	}

	// Set the response header and write the non-streaming response
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(resp.StatusCode)
	w.Write(respBody)
}
