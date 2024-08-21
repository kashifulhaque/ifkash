package handler

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"sort"
	"time"
)

type Story struct {
	By    string `json:"by"`
	ID    int    `json:"id"`
	Score int    `json:"score"`
	Time  int64  `json:"time"`
	Title string `json:"title"`
	Type  string `json:"type"`
	URL   string `json:"url"`
}

func fetchTopStories() ([]int, error) {
	resp, err := http.Get("https://hacker-news.firebaseio.com/v0/topstories.json")
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	var storyIDs []int
	err = json.Unmarshal(body, &storyIDs)
	if err != nil {
		return nil, err
	}

	return storyIDs, nil
}

func fetchStory(id int) (*Story, error) {
	url := fmt.Sprintf("https://hacker-news.firebaseio.com/v0/item/%d.json", id)
	resp, err := http.Get(url)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	var story Story
	err = json.Unmarshal(body, &story)
	if err != nil {
		return nil, err
	}

	return &story, nil
}

func HNHandler(w http.ResponseWriter, r *http.Request) {
	storyIDs, err := fetchTopStories()
	if err != nil {
		http.Error(w, "Error fetching top stories", http.StatusInternalServerError)
		return
	}

	var stories []*Story
	oneWeekAgo := time.Now().AddDate(0, 0, -7).Unix()

	for _, id := range storyIDs {
		story, err := fetchStory(id)
		if err != nil {
			continue
		}

		if story.Time >= oneWeekAgo {
			stories = append(stories, story)
		}

		if len(stories) >= 20 {
			break
		}
	}

	sort.Slice(stories, func(i, j int) bool {
		return stories[i].Score > stories[j].Score
	})

	if len(stories) > 20 {
		stories = stories[:20]
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(stories)
}
