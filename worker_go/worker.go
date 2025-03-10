package main

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
	"os"
	"sync"
	"time"

	"dagger.io/dagger"
	"github.com/gin-gonic/gin"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"go.mongodb.org/mongo-driver/v2/mongo"
	"go.mongodb.org/mongo-driver/v2/mongo/options"
)

// Task struct for incoming jobs
type Task struct {
	ID         int    `json:"id"`
	Repository string `json:"repo_path"`
	Task       string `json:"task"` // "rust-build", "rust-test", "python-lint", "code-review"
}

// TaskResult struct for returning task results
type TaskResult struct {
	ID     int    `json:"id"`
	Status string `json:"status"`
	Result string `json:"result"`
}

// Prometheus metrics
var (
	tasksProcessed = prometheus.NewCounter(
		prometheus.CounterOpts{
			Name: "tasks_processed_total",
			Help: "Total number of processed tasks",
		})

	rustBuildsTotal = prometheus.NewCounter(
		prometheus.CounterOpts{
			Name: "rust_build_total",
			Help: "Total number of Rust builds executed",
		})

	pythonLintsTotal = prometheus.NewCounter(
		prometheus.CounterOpts{
			Name: "python_lint_total",
			Help: "Total number of Python lint checks executed",
		})

	codeReviewsTotal = prometheus.NewCounter(
		prometheus.CounterOpts{
			Name: "code_review_total",
			Help: "Total number of code reviews executed",
		})

	mu sync.Mutex // Mutex to safely increment counters
)

func init() {
	// Register all Prometheus counters
	prometheus.MustRegister(tasksProcessed)
	prometheus.MustRegister(rustBuildsTotal)
	prometheus.MustRegister(pythonLintsTotal)
	prometheus.MustRegister(codeReviewsTotal)
}

// Execute a task based on type
func handleTask(c *gin.Context) {
	var task Task
	if err := c.ShouldBindJSON(&task); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request"})
		return
	}

	log.Printf("🚀 Received task: %+v\n", task)

	// Execute Task
	var result string
	var execErr error

	start := time.Now()

	switch task.Task {
	case "rust-build":
		result, execErr = runRustBuild(task.Repository)
		rustBuildsTotal.Inc()
	case "rust-build2":
		result = "✅ Rust Build Successful" // New simple response
		rustBuildsTotal.Inc()
	case "rust-test":
		result, execErr = runRustTests(task.Repository)
	case "python-lint":
		result, execErr = runPythonLint(task.Repository)
	case "python-lint2":
		result = "✅ Python Linting Successful" // New simple response
		pythonLintsTotal.Inc()
	case "code-review":
		result, execErr = runCodeReview(task.Repository) // New OpenAI Code Review
		codeReviewsTotal.Inc()
	default:
		execErr = fmt.Errorf("invalid task type: %s", task.Task)
	}

	duration := time.Since(start)

	// Check if task execution encountered an error
	if execErr != nil {
		log.Printf("❌ Task %d failed: %s", task.ID, execErr)
		c.JSON(http.StatusInternalServerError, gin.H{"error": execErr.Error()})
		return
	}

	// Write completion to MongoDB
	dbErr := writeTaskCompletionToDB(task, result)

	if dbErr != nil {
		log.Printf("❌ Failed to save task %d: %s", task.ID, dbErr)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to save task result"})
		return
	}

	log.Printf("✅ Task %d completed and saved", task.ID)
	// Increment processed task count
	mu.Lock()
	tasksProcessed.Inc()
	mu.Unlock()

	// Return response
	response := TaskResult{
		ID:     task.ID,
		Status: "completed",
		Result: fmt.Sprintf("%s (duration: %s)", result, duration),
	}

	c.JSON(http.StatusOK, response)
}

// Run Rust Build using Dagger
func runRustBuild(repoPath string) (string, error) {
	ctx := context.Background()
	client, err := dagger.Connect(ctx)
	if err != nil {
		return "", err
	}
	defer client.Close()

	// Mount project directory
	//projectDir := client.Host().Directory(repoPath)
	//shared := client.Host().Directory("/home/maso77/repos/distributed_cicd/shared")
	projectDir := client.Host().Directory("/client")
	shared := client.Host().Directory("/shared")
	ctr := client.Container().
		From("rust:latest").
		WithMountedDirectory("/app", projectDir).
		WithMountedDirectory("/shared", shared).
		WithWorkdir("/app").
		WithExec([]string{"cargo", "build", "--release"})

	_, err = ctr.ExitCode(ctx)
	if err != nil {
		return "", fmt.Errorf("Rust build failed: %w", err)
	}

	return "Rust build successful", nil
}

// Run Rust Tests using Dagger
func runRustTests(repoPath string) (string, error) {
	ctx := context.Background()
	client, err := dagger.Connect(ctx)
	if err != nil {
		return "", err
	}
	defer client.Close()

	//projectDir := client.Host().Directory(repoPath)
	//shared := client.Host().Directory("/home/maso77/repos/distributed_cicd/shared")
	projectDir := client.Host().Directory("/client")
	shared := client.Host().Directory("/shared")
	ctr := client.Container().
		From("rust:latest").
		WithMountedDirectory("/app", projectDir).
		WithMountedDirectory("/shared", shared).
		WithWorkdir("/app").
		WithExec([]string{"cargo", "test"})

	_, err = ctr.ExitCode(ctx)
	if err != nil {
		return "", fmt.Errorf("Rust tests failed: %w", err)
	}

	return "Rust tests passed", nil
}

func runPythonLint(repoPath string) (string, error) {
	ctx := context.Background()
	client, err := dagger.Connect(ctx)
	if err != nil {
		return "", fmt.Errorf("failed to connect to Dagger: %w", err)
	}
	defer client.Close()

	// Mount the project directory
	//projectDir := client.Host().Directory(repoPath)
	projectDir := client.Host().Directory("/python")
	// Step 1: Install Ruff
	base := client.Container().
		From("python:3.10").
		WithMountedDirectory("/app", projectDir).
		WithWorkdir("/app")

	install := base.WithExec([]string{"pip", "install", "--no-cache-dir", "ruff"})
	installOutput, installErr := install.Stdout(ctx)

	if installErr != nil {
		return "", fmt.Errorf("failed to install Ruff: %w\n%s", installErr, installOutput)
	}
	log.Printf("✅ Ruff installed successfully")

	// Step 2: Run Ruff check (FORCE EXIT ON FAILURE)
	lint := install.WithExec([]string{"sh", "-c", "ruff check . || exit 1"})

	// Capture output and exit code
	stdout, _ := lint.Stdout(ctx)
	stderr, _ := lint.Stderr(ctx)
	exitCode, exitErr := lint.ExitCode(ctx)

	if exitErr != nil {
		return "", fmt.Errorf("failed to retrieve Ruff exit code: %w", exitErr)
	}

	log.Printf("🐍 Ruff Exit Code: %d", exitCode)
	log.Printf("🐍 Ruff STDOUT:\n%s", stdout)
	log.Printf("🐍 Ruff STDERR:\n%s", stderr)

	// Combine stdout & stderr
	fullOutput := stdout + "\n" + stderr

	// If Ruff finds lint errors, force exit code to be 1
	if exitCode != 0 {
		errorCount := countRuffErrors(fullOutput)
		return fmt.Sprintf("❌ Ruff found %d issues:\n%s", errorCount, fullOutput), fmt.Errorf("ruff check failed")
	}

	return "✅ Python Ruff linting passed with 0 errors", nil
}

// Helper function to count Ruff errors
func countRuffErrors(output string) int {
	lines := bytes.Split([]byte(output), []byte("\n"))
	count := 0
	for _, line := range lines {
		if bytes.Contains(line, []byte("error")) { // Match "error" in Ruff output
			count++
		}
	}
	return count
}

// Run OpenAI Code Review
func runCodeReview(repoPath string) (string, error) {
	filePath := "/python/app.py" // Example file for testing

	code, err := ioutil.ReadFile(filePath)
	if err != nil {
		return "", fmt.Errorf("failed to read file: %w", err)
	}

	log.Println("📤 Sending code to OpenAI for review...")

	review, err := reviewCode(string(code))
	if err != nil {
		return "", fmt.Errorf("code review failed: %w", err)
	}

	return review, nil
}

// OpenAI API Integration
var openAIKey = "" // Replace with your API key

type OpenAIRequest struct {
	Model    string        `json:"model"`
	Messages []interface{} `json:"messages"`
}

type ChatMessage struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

func reviewCode(codeSnippet string) (string, error) {
	url := "https://api.openai.com/v1/chat/completions"

	messages := []interface{}{
		ChatMessage{"system", "You are a senior software engineer. Review the given code and give list of key improvements. You are shy person so you stick to short answers for example bullet points"},
		ChatMessage{"user", "Please review the following code:\n\n" + codeSnippet},
	}

	requestBody := OpenAIRequest{
		Model:    "gpt-4o-mini",
		Messages: messages,
	}

	jsonData, err := json.Marshal(requestBody)
	if err != nil {
		return "", err
	}

	req, err := http.NewRequest("POST", url, bytes.NewBuffer(jsonData))
	if err != nil {
		return "", err
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+openAIKey)

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	body, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}

	return string(body), nil
}

// Health check endpoint
func healthCheck(c *gin.Context) {
	c.String(http.StatusOK, "OK")
}

// Prometheus metrics endpoint
func metricsHandler() gin.HandlerFunc {
	return gin.WrapH(promhttp.Handler())
}

func main() {
	// Create a context
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	var err error

	// MongoDB URI
	mongoURI := os.Getenv("MONGODB_URI")
	client, err := mongo.Connect(options.Client().ApplyURI(mongoURI))

	if err != nil {
		log.Fatalf("Failed to connect to MongoDB: %v", err)
	}

	// Use a database and collection
	taskCollection = client.Database("taskdb").Collection("finished_tasks")

	// Ensure disconnection on main function return
	defer func() {
		if err = client.Disconnect(ctx); err != nil {
			panic(err)
		}
	}()

	port := os.Getenv("WORKER_PORT")
	if port == "" {
		port = "5001"
	}

	router := gin.Default()

	// Routes
	router.POST("/execute_task", handleTask)
	router.GET("/health", healthCheck)
	router.GET("/metrics", metricsHandler())

	addr := fmt.Sprintf("0.0.0.0:%s", port)
	log.Printf("🚀 Worker running on %s\n", addr)
	if err := router.Run(addr); err != nil {
		log.Fatalf("Failed to start server: %v", err)
	}
}

func writeTaskCompletionToDB(task Task, result string) error {
	finishedTask := FinishedTask{
		Task:       task,
		FinishTime: time.Now(),
		Result:     result,
	}

	// Insert the finished task into MongoDB
	_, err := taskCollection.InsertOne(context.TODO(), finishedTask)
	return err
}

type FinishedTask struct {
	Task       Task
	FinishTime time.Time
	Result     string
}

var client *mongo.Client
var taskCollection *mongo.Collection
