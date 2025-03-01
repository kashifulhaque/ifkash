package handler

import (
	"encoding/json"
	"net/http"
	"sort"
	"strings"
	"sync"
	"time"

	"github.com/mmcdole/gofeed"
)

type BlogPost struct {
	Title  string    `json:"title"`
	Link   string    `json:"link"`
	Date   time.Time `json:"date"`
	Author string    `json:"author"`
	Source string    `json:"source"`
}

type ApiResponse struct {
	Posts []BlogPost `json:"posts"`
	Blogs []string   `json:"blogs"`
}

type RSSSource struct {
	Name string
	URL  string
}

var rssSources = []RSSSource{
	{Name: "Netflix", URL: "https://netflixtechblog.com/feed"},
	{Name: "Meta", URL: "https://engineering.fb.com/feed/"},
	{Name: "Airbnb", URL: "https://medium.com/feed/airbnb-engineering"},
}

func fetchPosts(source RSSSource, wg *sync.WaitGroup, results chan<- []BlogPost, errors chan<- error) {
	defer wg.Done()

	fp := gofeed.NewParser()
	feed, err := fp.ParseURL(source.URL)
	if err != nil {
		errors <- err
		return
	}

	var posts []BlogPost
	for _, item := range feed.Items {
		author := ""
		if item.Author != nil {
			author = item.Author.Name
		}

		date := item.PublishedParsed
		if date == nil {
			date = item.UpdatedParsed
		}

		// Skip posts without dates
		if date == nil {
			continue
		}

		post := BlogPost{
			Title:  item.Title,
			Link:   item.Link,
			Date:   *date,
			Author: author,
			Source: source.Name,
		}
		posts = append(posts, post)
	}

	results <- posts
}

// Fetch posts from all sources in parallel
func getAllPosts() ([]BlogPost, error) {
	var wg sync.WaitGroup
	results := make(chan []BlogPost, len(rssSources))
	errors := make(chan error, len(rssSources))

	// Fetch posts from all sources concurrently
	for _, source := range rssSources {
		wg.Add(1)
		go fetchPosts(source, &wg, results, errors)
	}

	// Close channels when all goroutines complete
	go func() {
		wg.Wait()
		close(results)
		close(errors)
	}()

	var errs []error
	for err := range errors {
		errs = append(errs, err)
	}

	if len(errs) == len(rssSources) {
		return nil, errs[0]
	}

	var allPosts []BlogPost
	for posts := range results {
		allPosts = append(allPosts, posts...)
	}

	// Sort posts by date (newest first)
	sort.Slice(allPosts, func(i, j int) bool {
		return allPosts[i].Date.After(allPosts[j].Date)
	})

	return allPosts, nil
}

func getPostsBySource(sourceName string) ([]BlogPost, error) {
	for _, source := range rssSources {
		if source.Name == sourceName {
			var wg sync.WaitGroup
			results := make(chan []BlogPost, 1)
			errors := make(chan error, 1)

			wg.Add(1)
			go fetchPosts(source, &wg, results, errors)

			go func() {
				wg.Wait()
				close(results)
				close(errors)
			}()

			select {
			case err := <-errors:
				if err != nil {
					return nil, err
				}
			default:
			}

			posts := <-results
			return posts, nil
		}
	}

	return nil, nil
}

func Handler(w http.ResponseWriter, r *http.Request) {
	// Set CORS headers
	w.Header().Set("Access-Control-Allow-Origin", "*")
	w.Header().Set("Access-Control-Allow-Methods", "GET, OPTIONS")
	w.Header().Set("Access-Control-Allow-Headers", "Origin, Content-Type")
	
	// Handle preflight OPTIONS request
	if r.Method == "OPTIONS" {
		w.WriteHeader(http.StatusOK)
		return
	}
	
	// Only allow GET requests
	if r.Method != "GET" {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	
	// Set content type
	w.Header().Set("Content-Type", "application/json")
	
	// Get blog names for response
	blogNames := make([]string, len(rssSources))
	for i, source := range rssSources {
		blogNames[i] = source.Name
	}
	
	// Parse the path to check if a specific source is requested
	path := r.URL.Path
	pathParts := strings.Split(strings.TrimPrefix(path, "/"), "/")
	
	// If path has format /api/posts/{source}
	if len(pathParts) >= 3 && pathParts[0] == "api" && pathParts[1] == "posts" && pathParts[2] != "" {
		sourceName := pathParts[2]
		
		// Validate source name
		valid := false
		for _, source := range rssSources {
			if source.Name == sourceName {
				valid = true
				break
			}
		}
		
		if !valid {
			http.Error(w, "Invalid source", http.StatusBadRequest)
			return
		}
		
		posts, err := getPostsBySource(sourceName)
		if err != nil {
			http.Error(w, "Failed to fetch posts", http.StatusInternalServerError)
			return
		}
		
		response := ApiResponse{
			Posts: posts,
			Blogs: blogNames,
		}
		
		json.NewEncoder(w).Encode(response)
		return
	}
	
	// Default: return all posts
	posts, err := getAllPosts()
	if err != nil {
		http.Error(w, "Failed to fetch posts", http.StatusInternalServerError)
		return
	}
	
	response := ApiResponse{
		Posts: posts,
		Blogs: blogNames,
	}
	
	json.NewEncoder(w).Encode(response)
}
