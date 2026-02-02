package main

import (
	"encoding/json"
	"strings"
	"testing"
	"time"
)

func createTestComment(opts ...func(*PRComment)) PRComment {
	c := PRComment{
		ID:         1,
		FilePath:   "test.py",
		LineNumber: intPtr(10),
		StartLine:  nil,
		Author:     "reviewer",
		Body:       "Test comment",
		CreatedAt:  time.Date(2026, 1, 30, 10, 0, 0, 0, time.UTC),
		UpdatedAt:  time.Date(2026, 1, 30, 12, 0, 0, 0, time.UTC),
		DiffHunk:   "@@ -1,3 +1,5 @@\n    code here\n+   new code",
		HTMLURL:    "https://github.com/owner/repo/pull/1#discussion_r1",
	}
	for _, opt := range opts {
		opt(&c)
	}
	return c
}

func TestFormatCommentForLLM(t *testing.T) {
	t.Run("includes file and line", func(t *testing.T) {
		comment := createTestComment(func(c *PRComment) {
			c.FilePath = "src/main.py"
			c.LineNumber = intPtr(42)
		})
		output := FormatCommentForLLM(comment, true, 10)

		if !strings.Contains(output, "src/main.py") {
			t.Error("output missing file path")
		}
		if !strings.Contains(output, "line 42") {
			t.Error("output missing line number")
		}
	})

	t.Run("includes author", func(t *testing.T) {
		comment := createTestComment(func(c *PRComment) {
			c.Author = "reviewer1"
		})
		output := FormatCommentForLLM(comment, true, 10)

		if !strings.Contains(output, "reviewer1") {
			t.Error("output missing author")
		}
	})

	t.Run("includes date", func(t *testing.T) {
		comment := createTestComment()
		output := FormatCommentForLLM(comment, true, 10)

		if !strings.Contains(output, "2026-01-30") {
			t.Error("output missing date")
		}
	})

	t.Run("includes code snippet by default", func(t *testing.T) {
		comment := createTestComment(func(c *PRComment) {
			c.DiffHunk = "@@ -1,3 +1,5 @@\n    some code"
		})
		output := FormatCommentForLLM(comment, true, 10)

		if !strings.Contains(output, "```") {
			t.Error("output missing code block")
		}
		if !strings.Contains(output, "some code") {
			t.Error("output missing code content")
		}
	})

	t.Run("excludes snippet when disabled", func(t *testing.T) {
		comment := createTestComment(func(c *PRComment) {
			c.DiffHunk = "@@ -1,3 +1,5 @@\n    some code"
		})
		output := FormatCommentForLLM(comment, false, 10)

		if strings.Contains(output, "Code context") {
			t.Error("output should not contain Code context")
		}
	})

	t.Run("includes comment body", func(t *testing.T) {
		comment := createTestComment(func(c *PRComment) {
			c.Body = "Please fix this issue!"
		})
		output := FormatCommentForLLM(comment, true, 10)

		if !strings.Contains(output, "Please fix this issue!") {
			t.Error("output missing comment body")
		}
	})

	t.Run("handles empty diff hunk", func(t *testing.T) {
		comment := createTestComment(func(c *PRComment) {
			c.DiffHunk = ""
		})
		output := FormatCommentForLLM(comment, true, 10)

		if !strings.Contains(output, comment.FilePath) {
			t.Error("output missing file path")
		}
		if !strings.Contains(output, comment.Body) {
			t.Error("output missing body")
		}
	})
}

func TestFormatCommentsGrouped(t *testing.T) {
	t.Run("groups by file", func(t *testing.T) {
		comments := ParseComments(SampleComments)
		output := FormatCommentsGrouped(comments, true, 10)

		if !strings.Contains(output, "## src/main.py") {
			t.Error("output missing src/main.py header")
		}
		if !strings.Contains(output, "## src/processor.py") {
			t.Error("output missing src/processor.py header")
		}
	})

	t.Run("shows total count", func(t *testing.T) {
		comments := ParseComments(SampleComments)
		output := FormatCommentsGrouped(comments, true, 10)

		if !strings.Contains(output, "Total comments: 5") {
			t.Error("output missing total count")
		}
	})

	t.Run("shows file count", func(t *testing.T) {
		comments := ParseComments(SampleComments)
		output := FormatCommentsGrouped(comments, true, 10)

		if !strings.Contains(output, "Files with comments:") {
			t.Error("output missing file count")
		}
	})

	t.Run("empty comments", func(t *testing.T) {
		output := FormatCommentsGrouped([]PRComment{}, true, 10)

		if !strings.Contains(output, "No comments found") {
			t.Error("output should say no comments found")
		}
	})

	t.Run("respects snippet setting", func(t *testing.T) {
		comments := []PRComment{createTestComment()}
		output := FormatCommentsGrouped(comments, false, 10)

		if strings.Contains(output, "Code context") {
			t.Error("output should not contain Code context when disabled")
		}
	})
}

func TestFormatCommentsFlat(t *testing.T) {
	t.Run("includes numbered comments", func(t *testing.T) {
		comments := ParseComments(SampleComments)
		output := FormatCommentsFlat(comments, true, 10)

		if !strings.Contains(output, "## Comment 1") {
			t.Error("output missing Comment 1")
		}
		if !strings.Contains(output, "## Comment 2") {
			t.Error("output missing Comment 2")
		}
	})

	t.Run("shows total count", func(t *testing.T) {
		comments := ParseComments(SampleComments)
		output := FormatCommentsFlat(comments, true, 10)

		if !strings.Contains(output, "5 total") {
			t.Error("output missing total count")
		}
	})

	t.Run("empty comments", func(t *testing.T) {
		output := FormatCommentsFlat([]PRComment{}, true, 10)

		if !strings.Contains(output, "No comments found") {
			t.Error("output should say no comments found")
		}
	})
}

func TestFormatCommentsMinimal(t *testing.T) {
	t.Run("shows file path", func(t *testing.T) {
		comments := []PRComment{createTestComment(func(c *PRComment) {
			c.FilePath = "test.py"
		})}
		output := FormatCommentsMinimal(comments)

		if !strings.Contains(output, "test.py") {
			t.Error("output missing file path")
		}
	})

	t.Run("shows line info", func(t *testing.T) {
		comments := []PRComment{createTestComment(func(c *PRComment) {
			c.LineNumber = intPtr(42)
			c.Author = "bob"
		})}
		output := FormatCommentsMinimal(comments)

		if !strings.Contains(output, "line 42") {
			t.Error("output missing line info")
		}
		if !strings.Contains(output, "bob") {
			t.Error("output missing author")
		}
	})

	t.Run("truncates long bodies", func(t *testing.T) {
		longBody := strings.Repeat("x", 200)
		comments := []PRComment{createTestComment(func(c *PRComment) {
			c.Body = longBody
		})}
		output := FormatCommentsMinimal(comments)

		if !strings.Contains(output, "...") {
			t.Error("output should truncate with ellipsis")
		}
		if strings.Contains(output, longBody) {
			t.Error("output should not contain full body")
		}
	})

	t.Run("summary line", func(t *testing.T) {
		comments := ParseComments(SampleComments)
		output := FormatCommentsMinimal(comments)

		if !strings.Contains(output, "PR Comments:") {
			t.Error("output missing summary")
		}
		if !strings.Contains(output, "total") {
			t.Error("output missing total")
		}
		if !strings.Contains(output, "files") {
			t.Error("output missing files")
		}
	})

	t.Run("empty comments", func(t *testing.T) {
		output := FormatCommentsMinimal([]PRComment{})

		if !strings.Contains(output, "No comments found") {
			t.Error("output should say no comments found")
		}
	})
}

func TestFormatForClaude(t *testing.T) {
	t.Run("includes pr title", func(t *testing.T) {
		comments := []PRComment{createTestComment()}
		output := FormatForClaude(comments, "", "Fix authentication bug", true, 15)

		if !strings.Contains(output, "Fix authentication bug") {
			t.Error("output missing PR title")
		}
	})

	t.Run("includes pr url", func(t *testing.T) {
		comments := []PRComment{createTestComment()}
		output := FormatForClaude(comments, "https://github.com/owner/repo/pull/123", "", true, 15)

		if !strings.Contains(output, "https://github.com/owner/repo/pull/123") {
			t.Error("output missing PR URL")
		}
	})

	t.Run("includes header", func(t *testing.T) {
		comments := []PRComment{createTestComment()}
		output := FormatForClaude(comments, "", "", true, 15)

		if !strings.Contains(output, "Pull Request Review Comments") {
			t.Error("output missing header")
		}
	})

	t.Run("includes file info", func(t *testing.T) {
		comments := []PRComment{createTestComment(func(c *PRComment) {
			c.FilePath = "src/api/handler.py"
		})}
		output := FormatForClaude(comments, "", "", true, 15)

		if !strings.Contains(output, "File: `src/api/handler.py`") {
			t.Error("output missing file info")
		}
	})

	t.Run("includes instructions", func(t *testing.T) {
		comments := []PRComment{createTestComment()}
		output := FormatForClaude(comments, "", "", true, 15)

		if !strings.Contains(output, "Instructions for Addressing Comments") {
			t.Error("output missing instructions")
		}
		if !strings.Contains(strings.ToLower(output), "specific change") {
			t.Error("output missing specific change text")
		}
	})

	t.Run("includes reviewer info", func(t *testing.T) {
		comments := []PRComment{createTestComment(func(c *PRComment) {
			c.Author = "senior_dev"
		})}
		output := FormatForClaude(comments, "", "", true, 15)

		if !strings.Contains(output, "Reviewer:") {
			t.Error("output missing Reviewer label")
		}
		if !strings.Contains(output, "senior_dev") {
			t.Error("output missing reviewer name")
		}
	})

	t.Run("empty comments", func(t *testing.T) {
		output := FormatForClaude([]PRComment{}, "", "", true, 15)

		if !strings.Contains(output, "No review comments") {
			t.Error("output should say no review comments")
		}
	})

	t.Run("groups by file", func(t *testing.T) {
		comments := ParseComments(SampleComments)
		output := FormatForClaude(comments, "", "", true, 15)

		if !strings.Contains(output, "File: `src/main.py`") {
			t.Error("output missing src/main.py")
		}
		if !strings.Contains(output, "File: `src/processor.py`") {
			t.Error("output missing src/processor.py")
		}
	})

	t.Run("includes snippet by default", func(t *testing.T) {
		comments := []PRComment{createTestComment()}
		output := FormatForClaude(comments, "", "", true, 15)

		if !strings.Contains(output, "Code being reviewed:") {
			t.Error("output missing code section")
		}
		if !strings.Contains(output, "```") {
			t.Error("output missing code block")
		}
	})

	t.Run("can exclude snippets", func(t *testing.T) {
		comments := []PRComment{createTestComment()}
		output := FormatForClaude(comments, "", "", false, 15)

		if strings.Contains(output, "Code being reviewed:") {
			t.Error("output should not contain code section when disabled")
		}
	})
}

func TestFormatCommentsJSON(t *testing.T) {
	t.Run("valid json output", func(t *testing.T) {
		comments := ParseComments(SampleComments[:1])
		output := FormatCommentsJSON(comments, true, 15)

		var result []JSONComment
		err := json.Unmarshal([]byte(output), &result)
		if err != nil {
			t.Errorf("invalid JSON: %v", err)
		}
		if len(result) != 1 {
			t.Errorf("len = %v, want 1", len(result))
		}
	})

	t.Run("includes all fields", func(t *testing.T) {
		comments := ParseComments(SampleComments[:1])
		output := FormatCommentsJSON(comments, true, 15)

		var result []JSONComment
		json.Unmarshal([]byte(output), &result)

		if result[0].File != "src/main.py" {
			t.Errorf("File = %v, want src/main.py", result[0].File)
		}
		if result[0].Author != "reviewer1" {
			t.Errorf("Author = %v, want reviewer1", result[0].Author)
		}
	})

	t.Run("snippet can be excluded", func(t *testing.T) {
		comments := ParseComments(SampleComments[:1])
		output := FormatCommentsJSON(comments, false, 15)

		var result []JSONComment
		json.Unmarshal([]byte(output), &result)

		if result[0].Snippet != nil {
			t.Error("Snippet should be nil when excluded")
		}
	})

	t.Run("empty comments", func(t *testing.T) {
		output := FormatCommentsJSON([]PRComment{}, true, 15)

		if output != "[]" {
			t.Errorf("output = %v, want []", output)
		}
	})
}
