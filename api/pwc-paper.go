package handler

/// Imports
import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"

	"github.com/PuerkitoBio/goquery"
)

const baseUrl = "https://paperswithcode.com"

type PaperDetails struct {
	Title        string   `json:"title"`
	Authors      []Author `json:"authors"`
	DatasetURL   string   `json:"dataset_url"`
	ArxivPageURL string   `json:"arxiv_page_url"`
	ArxivPDFURL  string   `json:"arxiv_pdf_url"`
}

type Author struct {
	Name string `json:"name"`
	URL  string `json:"url"`
}

func scrapePaperDetails(paperURL string) (*PaperDetails, error) {
	res, err := http.Get(paperURL)
	if err != nil {
		return nil, err
	}
	defer res.Body.Close()

	doc, err := goquery.NewDocumentFromReader(res.Body)
	if err != nil {
		return nil, err
	}

	paper := &PaperDetails{}

	/// 1. Extract the title (from first h1 tag)
	paper.Title = strings.ReplaceAll(strings.ReplaceAll(doc.Find("h1").First().Text(), "\n", ""), " ", "")

	/// 2. Extract the list of authors
	doc.Find("div.authors span").Each(func(i int, s *goquery.Selection) {
		/// Skip the first span, which contains the date
		if i == 0 {
			return
		}

		author := Author{}
		authorLink := s.Find("a")
		if(len(authorLink.Text) == 0) {
			return
		}
		author.Name = authorLink.Text()

		authorURL, exists := authorLink.Attr("href")
		if exists {
			author.URL = baseUrl + authorURL
		}

		paper.Authors = append(paper.Authors, author)
	})

	/// 3. Extract the dataset URL
	datasetElement := doc.Find("div.paper-datasets a span").Parent()
	datasetURL, exists := datasetElement.Attr("href")
	if exists {
		paper.DatasetURL = baseUrl + datasetURL
	}

	/// 4. Extract the Arxiv page and PDF URLs
	doc.Find("a").Each(func(i int, s *goquery.Selection) {
		href, exists := s.Attr("href")
		if exists && strings.Contains(href, "arxiv.org") {
			if strings.HasSuffix(href, ".pdf") {
				paper.ArxivPDFURL = href
			} else {
				paper.ArxivPageURL = href
			}
		}
	})

	return paper, nil
}

func PWCPaperHandler(w http.ResponseWriter, r *http.Request) {
	queryParams := r.URL.Query()
	paperURL := queryParams.Get("paper_url")

	if paperURL == "" {
		http.Error(w, "Missing paper_url query parameter", http.StatusBadRequest)
		return
	}

	paperDetails, err := scrapePaperDetails(paperURL)
	if err != nil {
		http.Error(w, fmt.Sprintf("Error scraping paper: %v", err), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(paperDetails)
}
