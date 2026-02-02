package main

import (
	"time"
)

// ParseDateTime parses GitHub API datetime string to time.Time.
func ParseDateTime(dtStr string) time.Time {
	// GitHub uses ISO 8601 format: 2026-01-30T23:06:02Z
	t, err := time.Parse(time.RFC3339, dtStr)
	if err != nil {
		return time.Unix(0, 0)
	}
	return t
}

// ParseComment parses a single comment from GitHub API response.
func ParseComment(raw RawComment) PRComment {
	// Get line numbers - prefer the current line, fall back to original
	var lineNumber *int
	if raw.Line != nil {
		lineNumber = raw.Line
	} else if raw.OriginalLine != nil {
		lineNumber = raw.OriginalLine
	}

	var startLine *int
	if raw.StartLine != nil {
		startLine = raw.StartLine
	} else if raw.OriginalStartLine != nil {
		startLine = raw.OriginalStartLine
	}

	// Get author - handle both regular users and bots
	author := raw.User.Login
	if author == "" {
		author = "unknown"
	}

	filePath := raw.Path
	if filePath == "" {
		filePath = "unknown"
	}

	return PRComment{
		ID:         raw.ID,
		FilePath:   filePath,
		LineNumber: lineNumber,
		StartLine:  startLine,
		Author:     author,
		Body:       raw.Body,
		CreatedAt:  ParseDateTime(raw.CreatedAt),
		UpdatedAt:  ParseDateTime(raw.UpdatedAt),
		DiffHunk:   raw.DiffHunk,
		HTMLURL:    raw.HTMLURL,
	}
}

// ParseComments parses multiple comments from GitHub API response.
func ParseComments(rawComments []RawComment) []PRComment {
	comments := make([]PRComment, 0, len(rawComments))
	for _, raw := range rawComments {
		comments = append(comments, ParseComment(raw))
	}
	return comments
}

// FilterByAuthor filters comments by author.
func FilterByAuthor(comments []PRComment, author string) []PRComment {
	if author == "" {
		return comments
	}

	filtered := make([]PRComment, 0)
	for _, c := range comments {
		if c.Author == author {
			filtered = append(filtered, c)
		}
	}
	return filtered
}

// GetMostRecentPerFile gets the most recent comment for each file.
func GetMostRecentPerFile(comments []PRComment) []PRComment {
	fileComments := make(map[string]PRComment)

	for _, comment := range comments {
		existing, exists := fileComments[comment.FilePath]
		if !exists || comment.UpdatedAt.After(existing.UpdatedAt) {
			fileComments[comment.FilePath] = comment
		}
	}

	result := make([]PRComment, 0, len(fileComments))
	for _, comment := range fileComments {
		result = append(result, comment)
	}
	return result
}

// GroupByFile groups comments by file path.
func GroupByFile(comments []PRComment) map[string][]PRComment {
	grouped := make(map[string][]PRComment)

	for _, comment := range comments {
		grouped[comment.FilePath] = append(grouped[comment.FilePath], comment)
	}

	return grouped
}
