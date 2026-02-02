package main

import (
	"encoding/json"
	"fmt"
	"os/exec"
)

// GitHubAPIError represents an error from the GitHub API.
type GitHubAPIError struct {
	Message string
}

func (e *GitHubAPIError) Error() string {
	return e.Message
}

// RawComment represents the raw JSON structure from GitHub API.
type RawComment struct {
	ID                int    `json:"id"`
	Path              string `json:"path"`
	Line              *int   `json:"line"`
	OriginalLine      *int   `json:"original_line"`
	StartLine         *int   `json:"start_line"`
	OriginalStartLine *int   `json:"original_start_line"`
	Body              string `json:"body"`
	CreatedAt         string `json:"created_at"`
	UpdatedAt         string `json:"updated_at"`
	DiffHunk          string `json:"diff_hunk"`
	HTMLURL           string `json:"html_url"`
	User              struct {
		Login string `json:"login"`
	} `json:"user"`
}

// PRInfo represents basic PR information from GitHub API.
type PRInfo struct {
	HTMLURL string `json:"html_url"`
	Title   string `json:"title"`
}

// FetchPRComments fetches PR comments using the gh CLI.
func FetchPRComments(owner, repo string, prNumber int) ([]RawComment, error) {
	endpoint := fmt.Sprintf("repos/%s/%s/pulls/%d/comments", owner, repo, prNumber)

	cmd := exec.Command("gh", "api", endpoint)
	output, err := cmd.Output()
	if err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			return nil, &GitHubAPIError{
				Message: fmt.Sprintf("Failed to fetch PR comments: %s", string(exitErr.Stderr)),
			}
		}
		return nil, &GitHubAPIError{
			Message: fmt.Sprintf("Failed to fetch PR comments: %v", err),
		}
	}

	var comments []RawComment
	if err := json.Unmarshal(output, &comments); err != nil {
		return nil, &GitHubAPIError{
			Message: fmt.Sprintf("Failed to parse API response: %v", err),
		}
	}

	return comments, nil
}

// FetchPRInfo fetches basic PR information.
func FetchPRInfo(owner, repo string, prNumber int) (*PRInfo, error) {
	endpoint := fmt.Sprintf("repos/%s/%s/pulls/%d", owner, repo, prNumber)

	cmd := exec.Command("gh", "api", endpoint)
	output, err := cmd.Output()
	if err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			return nil, &GitHubAPIError{
				Message: fmt.Sprintf("Failed to fetch PR info: %s", string(exitErr.Stderr)),
			}
		}
		return nil, &GitHubAPIError{
			Message: fmt.Sprintf("Failed to fetch PR info: %v", err),
		}
	}

	var info PRInfo
	if err := json.Unmarshal(output, &info); err != nil {
		return nil, &GitHubAPIError{
			Message: fmt.Sprintf("Failed to parse API response: %v", err),
		}
	}

	return &info, nil
}
