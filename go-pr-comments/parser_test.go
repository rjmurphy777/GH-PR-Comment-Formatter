package main

import (
	"testing"
	"time"
)

func TestParseDateTime(t *testing.T) {
	tests := []struct {
		name    string
		input   string
		wantErr bool
		check   func(time.Time) bool
	}{
		{
			name:  "github datetime",
			input: "2026-01-30T10:00:00Z",
			check: func(t time.Time) bool {
				return t.Year() == 2026 && t.Month() == 1 && t.Day() == 30 &&
					t.Hour() == 10 && t.Minute() == 0 && t.Second() == 0
			},
		},
		{
			name:  "with seconds",
			input: "2026-01-30T10:30:45Z",
			check: func(t time.Time) bool {
				return t.Hour() == 10 && t.Minute() == 30 && t.Second() == 45
			},
		},
		{
			name:  "invalid returns epoch",
			input: "invalid",
			check: func(t time.Time) bool {
				return t.Unix() == 0
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := ParseDateTime(tt.input)
			if !tt.check(got) {
				t.Errorf("ParseDateTime(%q) = %v, check failed", tt.input, got)
			}
		})
	}
}

func TestParseComment(t *testing.T) {
	t.Run("minimal comment", func(t *testing.T) {
		comment := ParseComment(SampleCommentMinimal)

		if comment.ID != 123 {
			t.Errorf("ID = %v, want 123", comment.ID)
		}
		if comment.FilePath != "src/main.py" {
			t.Errorf("FilePath = %v, want src/main.py", comment.FilePath)
		}
		if *comment.LineNumber != 12 {
			t.Errorf("LineNumber = %v, want 12", *comment.LineNumber)
		}
		if comment.StartLine != nil {
			t.Errorf("StartLine = %v, want nil", comment.StartLine)
		}
		if comment.Author != "reviewer1" {
			t.Errorf("Author = %v, want reviewer1", comment.Author)
		}
		if comment.Body != "Consider adding a docstring here." {
			t.Errorf("Body = %v, want 'Consider adding a docstring here.'", comment.Body)
		}
	})

	t.Run("comment with line range", func(t *testing.T) {
		comment := ParseComment(SampleCommentWithRange)

		if comment.ID != 124 {
			t.Errorf("ID = %v, want 124", comment.ID)
		}
		if *comment.LineNumber != 30 {
			t.Errorf("LineNumber = %v, want 30", *comment.LineNumber)
		}
		if *comment.StartLine != 25 {
			t.Errorf("StartLine = %v, want 25", *comment.StartLine)
		}
	})

	t.Run("bot comment", func(t *testing.T) {
		comment := ParseComment(SampleCommentBot)

		if comment.Author != "devin-ai-integration[bot]" {
			t.Errorf("Author = %v, want devin-ai-integration[bot]", comment.Author)
		}
	})

	t.Run("comment no line number", func(t *testing.T) {
		comment := ParseComment(SampleCommentNoLine)

		if comment.LineNumber != nil {
			t.Errorf("LineNumber = %v, want nil", comment.LineNumber)
		}
		if comment.FilePath != "README.md" {
			t.Errorf("FilePath = %v, want README.md", comment.FilePath)
		}
	})

	t.Run("falls back to original_line", func(t *testing.T) {
		comment := ParseComment(SampleCommentComplex)

		if *comment.LineNumber != 3376 {
			t.Errorf("LineNumber = %v, want 3376 (from original_line)", *comment.LineNumber)
		}
		if *comment.StartLine != 3374 {
			t.Errorf("StartLine = %v, want 3374 (from original_start_line)", *comment.StartLine)
		}
	})

	t.Run("handles missing user", func(t *testing.T) {
		raw := SampleCommentMinimal
		raw.User.Login = ""
		comment := ParseComment(raw)

		if comment.Author != "unknown" {
			t.Errorf("Author = %v, want unknown", comment.Author)
		}
	})

	t.Run("preserves html url", func(t *testing.T) {
		comment := ParseComment(SampleCommentMinimal)

		if comment.HTMLURL != "https://github.com/ROKT/canal/pull/123#discussion_r123" {
			t.Errorf("HTMLURL = %v, want https://github.com/ROKT/canal/pull/123#discussion_r123", comment.HTMLURL)
		}
	})
}

func TestParseComments(t *testing.T) {
	t.Run("empty list", func(t *testing.T) {
		result := ParseComments([]RawComment{})
		if len(result) != 0 {
			t.Errorf("len = %v, want 0", len(result))
		}
	})

	t.Run("multiple comments", func(t *testing.T) {
		result := ParseComments(SampleComments)
		if len(result) != 5 {
			t.Errorf("len = %v, want 5", len(result))
		}
	})

	t.Run("preserves order", func(t *testing.T) {
		result := ParseComments(SampleComments)
		if result[0].ID != 123 {
			t.Errorf("result[0].ID = %v, want 123", result[0].ID)
		}
		if result[1].ID != 124 {
			t.Errorf("result[1].ID = %v, want 124", result[1].ID)
		}
		if result[2].ID != 125 {
			t.Errorf("result[2].ID = %v, want 125", result[2].ID)
		}
	})
}

func TestFilterByAuthor(t *testing.T) {
	comments := ParseComments(SampleComments)

	t.Run("returns all when empty author", func(t *testing.T) {
		result := FilterByAuthor(comments, "")
		if len(result) != len(comments) {
			t.Errorf("len = %v, want %v", len(result), len(comments))
		}
	})

	t.Run("filter by specific author", func(t *testing.T) {
		result := FilterByAuthor(comments, "reviewer1")
		if len(result) != 2 {
			t.Errorf("len = %v, want 2", len(result))
		}
		for _, c := range result {
			if c.Author != "reviewer1" {
				t.Errorf("Author = %v, want reviewer1", c.Author)
			}
		}
	})

	t.Run("filter by bot author", func(t *testing.T) {
		result := FilterByAuthor(comments, "devin-ai-integration[bot]")
		if len(result) != 1 {
			t.Errorf("len = %v, want 1", len(result))
		}
		if result[0].Author != "devin-ai-integration[bot]" {
			t.Errorf("Author = %v, want devin-ai-integration[bot]", result[0].Author)
		}
	})

	t.Run("filter by nonexistent author", func(t *testing.T) {
		result := FilterByAuthor(comments, "nonexistent_user")
		if len(result) != 0 {
			t.Errorf("len = %v, want 0", len(result))
		}
	})
}

func TestGetMostRecentPerFile(t *testing.T) {
	t.Run("single comment per file", func(t *testing.T) {
		comments := ParseComments([]RawComment{SampleCommentMinimal, SampleCommentWithRange})
		result := GetMostRecentPerFile(comments)

		if len(result) != 2 {
			t.Errorf("len = %v, want 2", len(result))
		}

		files := make(map[string]bool)
		for _, c := range result {
			files[c.FilePath] = true
		}
		if !files["src/main.py"] {
			t.Error("missing src/main.py")
		}
		if !files["src/processor.py"] {
			t.Error("missing src/processor.py")
		}
	})

	t.Run("multiple comments same file", func(t *testing.T) {
		comments := ParseComments([]RawComment{SampleCommentMinimal, SampleCommentSameFile})
		result := GetMostRecentPerFile(comments)

		if len(result) != 1 {
			t.Errorf("len = %v, want 1", len(result))
		}
		if result[0].FilePath != "src/main.py" {
			t.Errorf("FilePath = %v, want src/main.py", result[0].FilePath)
		}
		// SampleCommentSameFile is more recent (15:30 vs 10:00)
		if result[0].ID != 127 {
			t.Errorf("ID = %v, want 127 (most recent)", result[0].ID)
		}
	})

	t.Run("empty list", func(t *testing.T) {
		result := GetMostRecentPerFile([]PRComment{})
		if len(result) != 0 {
			t.Errorf("len = %v, want 0", len(result))
		}
	})
}

func TestGroupByFile(t *testing.T) {
	t.Run("groups comments", func(t *testing.T) {
		comments := ParseComments(SampleComments)
		grouped := GroupByFile(comments)

		if len(grouped["src/main.py"]) != 2 {
			t.Errorf("src/main.py count = %v, want 2", len(grouped["src/main.py"]))
		}
		if len(grouped["src/processor.py"]) != 1 {
			t.Errorf("src/processor.py count = %v, want 1", len(grouped["src/processor.py"]))
		}
	})

	t.Run("empty list", func(t *testing.T) {
		grouped := GroupByFile([]PRComment{})
		if len(grouped) != 0 {
			t.Errorf("len = %v, want 0", len(grouped))
		}
	})

	t.Run("single file", func(t *testing.T) {
		comments := ParseComments([]RawComment{SampleCommentMinimal, SampleCommentSameFile})
		grouped := GroupByFile(comments)

		if len(grouped) != 1 {
			t.Errorf("len = %v, want 1", len(grouped))
		}
		if len(grouped["src/main.py"]) != 2 {
			t.Errorf("src/main.py count = %v, want 2", len(grouped["src/main.py"]))
		}
	})
}
