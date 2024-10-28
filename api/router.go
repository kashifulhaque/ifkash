package handler

import (
	"github.com/gin-gonic/gin"
	"github.com/kashifulhaque/ifkash/api/handlers"
)

func SetupRouter(r *gin.Engine) {
	r.Use(gin.Logger())
	r.Use(gin.Recovery())

	api := r.Group("/api")
	{
		api.GET("/hn", handlers.GetHNStories)
		// api.GET("/pwc", handlers.GetPWCPapers)
		// api.GET("/pwc/paper", handlers.GetPWCPaperDetails)
		// api.GET("/leetcode", handlers.GetLeetCodeStats)
		// api.POST("/llm", handlers.ProxyLLMRequest)
	}
}
