package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/kashifulhaque/ifkash/api/services"
)

func GetHNStories(c *gin.Context) {
	hnService := services.NewHNService()
	
	stories, err := hnService.GetTopStories()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "Error fetching top stories",
		})
		return
	}

	c.JSON(http.StatusOK, stories)
}