package main

import (
	"encoding/json"
	"fmt"
	"sort"
	"strings"
)

// FormatCommentForLLM formats a single comment for LLM consumption.
func FormatCommentForLLM(comment PRComment, includeSnippet bool, snippetLines int) string {
	var lines []string

	// Header with file and line info
	lines = append(lines, fmt.Sprintf("### %s (%s)", comment.FilePath, comment.GetLineInfo()))
	lines = append(lines, fmt.Sprintf("**Author:** %s", comment.Author))
	lines = append(lines, fmt.Sprintf("**Date:** %s", comment.UpdatedAt.UTC().Format("2006-01-02 15:04 UTC")))
	lines = append(lines, "")

	// Code snippet
	if includeSnippet {
		snippet := comment.GetCodeSnippet(snippetLines)
		if snippet != "" {
			lines = append(lines, "**Code context:**")
			lines = append(lines, "```")
			lines = append(lines, snippet)
			lines = append(lines, "```")
			lines = append(lines, "")
		}
	}

	// Comment body
	lines = append(lines, "**Comment:**")
	lines = append(lines, comment.Body)
	lines = append(lines, "")

	return strings.Join(lines, "\n")
}

// FormatCommentsGrouped formats comments grouped by file.
func FormatCommentsGrouped(comments []PRComment, includeSnippet bool, snippetLines int) string {
	if len(comments) == 0 {
		return "No comments found."
	}

	grouped := GroupByFile(comments)
	var outputLines []string

	outputLines = append(outputLines, "# PR Comments Summary")
	outputLines = append(outputLines, fmt.Sprintf("Total comments: %d", len(comments)))
	outputLines = append(outputLines, fmt.Sprintf("Files with comments: %d", len(grouped)))
	outputLines = append(outputLines, "")

	// Sort file paths for consistent output
	filePaths := make([]string, 0, len(grouped))
	for path := range grouped {
		filePaths = append(filePaths, path)
	}
	sort.Strings(filePaths)

	for _, filePath := range filePaths {
		fileComments := grouped[filePath]
		// Sort by line number, then by date
		sort.Slice(fileComments, func(i, j int) bool {
			lineI := fileComments[i].GetLineNumberValue()
			lineJ := fileComments[j].GetLineNumberValue()
			if lineI != lineJ {
				return lineI < lineJ
			}
			return fileComments[i].UpdatedAt.Before(fileComments[j].UpdatedAt)
		})

		outputLines = append(outputLines, fmt.Sprintf("## %s", filePath))
		outputLines = append(outputLines, fmt.Sprintf("(%d comment(s))", len(fileComments)))
		outputLines = append(outputLines, "")

		for _, comment := range fileComments {
			outputLines = append(outputLines, FormatCommentForLLM(comment, includeSnippet, snippetLines))
			outputLines = append(outputLines, "---")
			outputLines = append(outputLines, "")
		}
	}

	return strings.Join(outputLines, "\n")
}

// FormatCommentsFlat formats comments in a flat list, sorted by date.
func FormatCommentsFlat(comments []PRComment, includeSnippet bool, snippetLines int) string {
	if len(comments) == 0 {
		return "No comments found."
	}

	// Sort by date, most recent first
	sortedComments := make([]PRComment, len(comments))
	copy(sortedComments, comments)
	sort.Slice(sortedComments, func(i, j int) bool {
		return sortedComments[i].UpdatedAt.After(sortedComments[j].UpdatedAt)
	})

	var outputLines []string
	outputLines = append(outputLines, fmt.Sprintf("# PR Comments (%d total)", len(comments)))
	outputLines = append(outputLines, "")

	for i, comment := range sortedComments {
		outputLines = append(outputLines, fmt.Sprintf("## Comment %d", i+1))
		outputLines = append(outputLines, FormatCommentForLLM(comment, includeSnippet, snippetLines))
		outputLines = append(outputLines, "---")
		outputLines = append(outputLines, "")
	}

	return strings.Join(outputLines, "\n")
}

// FormatCommentsMinimal formats comments in a minimal format for quick overview.
func FormatCommentsMinimal(comments []PRComment) string {
	if len(comments) == 0 {
		return "No comments found."
	}

	grouped := GroupByFile(comments)
	var outputLines []string

	outputLines = append(outputLines, fmt.Sprintf("PR Comments: %d total across %d files", len(comments), len(grouped)))
	outputLines = append(outputLines, "")

	// Sort file paths for consistent output
	filePaths := make([]string, 0, len(grouped))
	for path := range grouped {
		filePaths = append(filePaths, path)
	}
	sort.Strings(filePaths)

	for _, filePath := range filePaths {
		fileComments := grouped[filePath]
		outputLines = append(outputLines, fmt.Sprintf("  %s", filePath))

		// Sort by line number
		sort.Slice(fileComments, func(i, j int) bool {
			return fileComments[i].GetLineNumberValue() < fileComments[j].GetLineNumberValue()
		})

		for _, comment := range fileComments {
			// Truncate body for preview
			bodyPreview := strings.ReplaceAll(comment.Body, "\n", " ")
			if len(bodyPreview) > 100 {
				bodyPreview = bodyPreview[:100] + "..."
			}

			outputLines = append(outputLines, fmt.Sprintf("    - %s (%s): %s", comment.GetLineInfo(), comment.Author, bodyPreview))
		}

		outputLines = append(outputLines, "")
	}

	return strings.Join(outputLines, "\n")
}

// FormatForClaude formats comments specifically optimized for Claude/LLM consumption.
func FormatForClaude(comments []PRComment, prURL, prTitle string, includeSnippet bool, snippetLines int) string {
	if len(comments) == 0 {
		return "No review comments found on this PR."
	}

	grouped := GroupByFile(comments)
	var lines []string

	// Header
	lines = append(lines, "# Pull Request Review Comments")
	if prTitle != "" {
		lines = append(lines, fmt.Sprintf("**PR Title:** %s", prTitle))
	}
	if prURL != "" {
		lines = append(lines, fmt.Sprintf("**PR URL:** %s", prURL))
	}
	lines = append(lines, fmt.Sprintf("**Total Comments:** %d", len(comments)))
	lines = append(lines, fmt.Sprintf("**Files Affected:** %d", len(grouped)))
	lines = append(lines, "")
	lines = append(lines, "Below are the review comments that need to be addressed. Each comment includes:")
	lines = append(lines, "- The file path and line number(s)")
	lines = append(lines, "- A code snippet showing the context")
	lines = append(lines, "- The reviewer's comment/feedback")
	lines = append(lines, "")
	lines = append(lines, "---")
	lines = append(lines, "")

	// Sort file paths for consistent output
	filePaths := make([]string, 0, len(grouped))
	for path := range grouped {
		filePaths = append(filePaths, path)
	}
	sort.Strings(filePaths)

	for _, filePath := range filePaths {
		fileComments := grouped[filePath]
		sort.Slice(fileComments, func(i, j int) bool {
			lineI := fileComments[i].GetLineNumberValue()
			lineJ := fileComments[j].GetLineNumberValue()
			if lineI != lineJ {
				return lineI < lineJ
			}
			return fileComments[i].UpdatedAt.Before(fileComments[j].UpdatedAt)
		})

		lines = append(lines, fmt.Sprintf("## File: `%s`", filePath))
		lines = append(lines, "")

		for _, comment := range fileComments {
			lines = append(lines, fmt.Sprintf("### %s", comment.GetLineInfo()))
			lines = append(lines, fmt.Sprintf("**Reviewer:** %s", comment.Author))
			lines = append(lines, "")

			if includeSnippet {
				snippet := comment.GetCodeSnippet(snippetLines)
				if snippet != "" {
					lines = append(lines, "**Code being reviewed:**")
					lines = append(lines, "```")
					lines = append(lines, snippet)
					lines = append(lines, "```")
					lines = append(lines, "")
				}
			}

			lines = append(lines, "**Review comment:**")
			lines = append(lines, comment.Body)
			lines = append(lines, "")
			lines = append(lines, "---")
			lines = append(lines, "")
		}
	}

	// Footer with instructions
	lines = append(lines, "## Instructions for Addressing Comments")
	lines = append(lines, "")
	lines = append(lines, "Please review each comment above and make the necessary changes to the code.")
	lines = append(lines, "For each comment, consider:")
	lines = append(lines, "1. What specific change is being requested?")
	lines = append(lines, "2. Is the suggestion valid and should be implemented?")
	lines = append(lines, "3. Are there any related changes needed in other parts of the codebase?")

	return strings.Join(lines, "\n")
}

// JSONComment represents the JSON output format for a comment.
type JSONComment struct {
	File    string  `json:"file"`
	Line    *int    `json:"line"`
	Author  string  `json:"author"`
	Body    string  `json:"body"`
	Snippet *string `json:"snippet"`
	URL     string  `json:"url"`
}

// FormatCommentsJSON formats comments as JSON.
func FormatCommentsJSON(comments []PRComment, includeSnippet bool, snippetLines int) string {
	jsonComments := make([]JSONComment, 0, len(comments))

	for _, c := range comments {
		jc := JSONComment{
			File:   c.FilePath,
			Line:   c.LineNumber,
			Author: c.Author,
			Body:   c.Body,
			URL:    c.HTMLURL,
		}

		if includeSnippet {
			snippet := c.GetCodeSnippet(snippetLines)
			if snippet != "" {
				jc.Snippet = &snippet
			}
		}

		jsonComments = append(jsonComments, jc)
	}

	output, err := json.MarshalIndent(jsonComments, "", "  ")
	if err != nil {
		return "[]"
	}

	return string(output)
}
