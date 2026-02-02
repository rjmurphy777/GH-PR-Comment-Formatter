package main

import (
	"strings"
	"testing"
	"time"
)

func TestPRComment_GetLineInfo(t *testing.T) {
	tests := []struct {
		name       string
		lineNumber *int
		startLine  *int
		want       string
	}{
		{
			name:       "single line",
			lineNumber: intPtr(42),
			startLine:  nil,
			want:       "line 42",
		},
		{
			name:       "line range",
			lineNumber: intPtr(50),
			startLine:  intPtr(45),
			want:       "lines 45-50",
		},
		{
			name:       "same start and end",
			lineNumber: intPtr(42),
			startLine:  intPtr(42),
			want:       "line 42",
		},
		{
			name:       "no line number",
			lineNumber: nil,
			startLine:  nil,
			want:       "line unknown",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &PRComment{
				LineNumber: tt.lineNumber,
				StartLine:  tt.startLine,
			}
			if got := c.GetLineInfo(); got != tt.want {
				t.Errorf("GetLineInfo() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestPRComment_GetCodeSnippet(t *testing.T) {
	tests := []struct {
		name     string
		diffHunk string
		maxLines int
		wantContains   []string
		wantNotContains []string
		wantEmpty bool
	}{
		{
			name:      "empty diff",
			diffHunk:  "",
			maxLines:  10,
			wantEmpty: true,
		},
		{
			name:         "removes header",
			diffHunk:     "@@ -10,3 +10,5 @@ def hello():\n     print('hello')\n+    print('world')",
			maxLines:     10,
			wantContains: []string{"print('hello')", "print('world')"},
			wantNotContains: []string{"@@"},
		},
		{
			name:     "truncates long diff",
			diffHunk: "@@ -1,20 +1,20 @@ header\n    line 0\n    line 1\n    line 2\n    line 3\n    line 4\n    line 5\n    line 6\n    line 7\n    line 8\n    line 9",
			maxLines: 5,
			wantContains: []string{"line 5", "line 6", "line 7", "line 8", "line 9"},
			wantNotContains: []string{"line 0", "line 1"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &PRComment{DiffHunk: tt.diffHunk}
			got := c.GetCodeSnippet(tt.maxLines)

			if tt.wantEmpty && got != "" {
				t.Errorf("GetCodeSnippet() = %v, want empty", got)
			}

			for _, want := range tt.wantContains {
				if !strings.Contains(got, want) {
					t.Errorf("GetCodeSnippet() missing %q", want)
				}
			}

			for _, notWant := range tt.wantNotContains {
				if strings.Contains(got, notWant) {
					t.Errorf("GetCodeSnippet() should not contain %q", notWant)
				}
			}
		})
	}
}

func TestPRComment_GetLineNumberValue(t *testing.T) {
	tests := []struct {
		name       string
		lineNumber *int
		want       int
	}{
		{
			name:       "with value",
			lineNumber: intPtr(42),
			want:       42,
		},
		{
			name:       "nil",
			lineNumber: nil,
			want:       0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &PRComment{LineNumber: tt.lineNumber}
			if got := c.GetLineNumberValue(); got != tt.want {
				t.Errorf("GetLineNumberValue() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestPRComment_Create(t *testing.T) {
	now := time.Now()
	comment := PRComment{
		ID:         123,
		FilePath:   "src/main.py",
		LineNumber: intPtr(42),
		StartLine:  nil,
		Author:     "reviewer1",
		Body:       "This is a comment",
		CreatedAt:  now,
		UpdatedAt:  now,
		DiffHunk:   "@@ -10,3 +10,5 @@\n     print('hello')\n+    print('world')",
		HTMLURL:    "https://github.com/owner/repo/pull/1#discussion_r123",
	}

	if comment.ID != 123 {
		t.Errorf("ID = %v, want 123", comment.ID)
	}
	if comment.FilePath != "src/main.py" {
		t.Errorf("FilePath = %v, want src/main.py", comment.FilePath)
	}
	if *comment.LineNumber != 42 {
		t.Errorf("LineNumber = %v, want 42", *comment.LineNumber)
	}
	if comment.Author != "reviewer1" {
		t.Errorf("Author = %v, want reviewer1", comment.Author)
	}
}
