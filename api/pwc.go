package handler

/// Imports
import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"

	"github.com/PuerkitoBio/goquery"
)

/// Constants
const baseURL = "https://paperswithcode.com"

/// Enums
type Paper struct {
	Stars				string `json:"stars"`
	Slug        string `json:"slug"`
	PaperURL    string `json:"paper_url"`
	CodeURL     string `json:"code_url"`
	ImageURL    string `json:"image_url"`
	Title       string `json:"title"`
	GithubRepo  string `json:"github_repo"`
	Framework   string `json:"framework"`
	PublishDate string `json:"publish_date"`
	Description string `json:"description"`
	Tags        []Tag  `json:"tags"`
}

type Tag struct {
	Name string `json:"name"`
	URL  string `json:"url"`
}

/// Helpers
func scrapePapers(url string) ([]Paper, error) {
	res, err := http.Get(url)
	if err != nil {
		return nil, err
	}
	defer res.Body.Close()

	doc, err := goquery.NewDocumentFromReader(res.Body)
	if err != nil {
		return nil, err
	}

	var papers []Paper

	/// Go through each card (a card has a paper) on the home page
	doc.Find("div.infinite-container.text-center.home-page > div.row.infinite-item.item.paper-card").Each(func(i int, s *goquery.Selection) {
		paper := Paper{}

		/// Extract URL for the paper and append "#code" at the end to get paper's code URL
		paperURL, exists := s.Find("h1 a").Attr("href")
		if exists {
			paper.Slug = paperURL
			paper.PaperURL = baseUrl + paperURL
			paper.CodeURL = paper.PaperURL + "#code"
		}

		/// Extract any image URLs if they exist
		imageURL, exists := s.Find("div.item-image").Attr("style")
		if exists {
			paper.ImageURL = extractImageURL(imageURL)
		}

		/// Extract the paper's title
		paper.Title = s.Find("h1 a").Text()

		/// Extract the number of stars
		paper.Stars = s.Find("span.badge.badge-secondary").Text()

		/// Extract the GitHub repo for that paper, if it exists
		githubRepo, exists := s.Find("span.item-github-link a").Attr("href")
		if exists {
			paper.GithubRepo = githubRepo
		}

		/// Extract the framework used to implement that paper e.g. PyTorch, JAX, etc.
		frameworkImg, exists := s.Find("span.item-framework-link img").Attr("src")
		if exists {
			paper.Framework = frameworkImg
		}

		/// Extract publishing date of the paper
		paper.PublishDate = s.Find("span.author-name-text.item-date-pub").Text()

		/// Extract the short description of that paper
		paper.Description = s.Find("p.item-strip-abstract").Text()

		/// Extract tags associated with that paper and it's URLs
		s.Find("p > a").Each(func(i int, tagSel *goquery.Selection) {
			tagURL, exists := tagSel.Attr("href")
			if exists {
				tag := Tag{
					Name: tagSel.Find("span.badge span:last-child").Text(),
					URL:  baseUrl + tagURL,
				}
				paper.Tags = append(paper.Tags, tag)
			}
		})

		papers = append(papers, paper)
	})

	return papers, nil
}

func extractImageURL(style string) string {
	start := strings.Index(style, "url('") + 5
	end := strings.Index(style[start:], "'")
	if start > 4 && end > 0 {
		return style[start : start+end]
	}
	return ""
}

// / Entrypoint
func PWCHandler(w http.ResponseWriter, r *http.Request) {
	papers, err := scrapePapers(baseUrl)
	if err != nil {
		http.Error(w, fmt.Sprintf("Error scraping papers: %v", err), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(papers)
}
