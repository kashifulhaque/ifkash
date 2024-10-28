package handler

import (
	"net/http"
	"os"

	"github.com/gin-gonic/gin"
)

func Handler(w http.ResponseWriter, r *http.Request) {
	if os.Getenv("VERCEL_ENV") == "production" {
		gin.SetMode(gin.ReleaseMode)
	}

	app := gin.New()
	SetupRouter(app)
	app.ServeHTTP(w, r)
}
