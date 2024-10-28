package models

type Story struct {
	By    string `json:"by"`
	ID    int    `json:"id"`
	Score int    `json:"score"`
	Time  int64  `json:"time"`
	Title string `json:"title"`
	Type  string `json:"type"`
	URL   string `json:"url"`
}
