package main

import (
	"fmt"
	"strings"
	"time"
)

// PRComment represents a parsed PR comment with essential information.
type PRComment struct {
	ID         int       `json:"id"`
	FilePath   string    `json:"path"`
	LineNumber *int      `json:"line"`
	StartLine  *int      `json:"start_line"`
	Author     string    `json:"author"`
	Body       string    `json:"body"`
	CreatedAt  time.Time `json:"created_at"`
	UpdatedAt  time.Time `json:"updated_at"`
	DiffHunk   string    `json:"diff_hunk"`
	HTMLURL    string    `json:"html_url"`
}

// GetCodeSnippet extracts a relevant code snippet from the diff hunk.
func (c *PRComment) GetCodeSnippet(maxLines int) string {
	if c.DiffHunk == "" {
		return ""
	}

	lines := strings.Split(c.DiffHunk, "\n")

	// Remove the diff header line (starts with @@)
	var contentLines []string
	for _, line := range lines {
		if !strings.HasPrefix(line, "@@") {
			contentLines = append(contentLines, line)
		}
	}

	if len(contentLines) <= maxLines {
		return strings.Join(contentLines, "\n")
	}

	// Take the last maxLines lines (most relevant to the comment)
	return strings.Join(contentLines[len(contentLines)-maxLines:], "\n")
}

// GetLineInfo returns a human-readable line number string.
func (c *PRComment) GetLineInfo() string {
	if c.StartLine != nil && c.LineNumber != nil && *c.StartLine != *c.LineNumber {
		return fmt.Sprintf("lines %d-%d", *c.StartLine, *c.LineNumber)
	} else if c.LineNumber != nil {
		return fmt.Sprintf("line %d", *c.LineNumber)
	}
	return "line unknown"
}

// GetLineNumberValue returns the line number or 0 if nil.
func (c *PRComment) GetLineNumberValue() int {
	if c.LineNumber != nil {
		return *c.LineNumber
	}
	return 0
}
