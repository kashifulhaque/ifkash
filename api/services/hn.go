package services

import (
	"encoding/json"
	"fmt"
	"sort"
	"time"

	"github.com/kashifulhaque/ifkash/api/models"
	"github.com/kashifulhaque/ifkash/api/utils"
)

type HNService struct{}

func NewHNService() *HNService {
	return &HNService{}
}

func (s *HNService) FetchTopStories() ([]int, error) {
	body, err := utils.FetchURL("https://hacker-news.firebaseio.com/v0/topstories.json")
	if err != nil {
		return nil, err
	}

	var storyIDs []int
	err = json.Unmarshal(body, &storyIDs)
	return storyIDs, err
}

func (s *HNService) FetchStory(id int) (*models.Story, error) {
	url := fmt.Sprintf("https://hacker-news.firebaseio.com/v0/item/%d.json", id)
	body, err := utils.FetchURL(url)
	if err != nil {
		return nil, err
	}

	var story models.Story
	err = json.Unmarshal(body, &story)
	return &story, err
}

func (s *HNService) GetTopStories() ([]*models.Story, error) {
	storyIDs, err := s.FetchTopStories()
	if err != nil {
		return nil, err
	}

	var stories []*models.Story
	oneWeekAgo := time.Now().AddDate(0, 0, -7).Unix()

	for _, id := range storyIDs {
		story, err := s.FetchStory(id)
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

	return stories, nil
}
