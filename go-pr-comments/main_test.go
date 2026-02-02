package main

import (
	"testing"
)

func TestParsePRURL(t *testing.T) {
	tests := []struct {
		name      string
		url       string
		wantOwner string
		wantRepo  string
		wantPR    int
		wantErr   bool
	}{
		{
			name:      "full url",
			url:       "https://github.com/ROKT/canal/pull/14777",
			wantOwner: "ROKT",
			wantRepo:  "canal",
			wantPR:    14777,
		},
		{
			name:      "url with trailing slash",
			url:       "https://github.com/owner/repo/pull/123/",
			wantOwner: "owner",
			wantRepo:  "repo",
			wantPR:    123,
		},
		{
			name:      "shorthand format",
			url:       "ROKT/canal#14777",
			wantOwner: "ROKT",
			wantRepo:  "canal",
			wantPR:    14777,
		},
		{
			name:    "invalid url",
			url:     "not-a-url",
			wantErr: true,
		},
		{
			name:    "non-pr github url",
			url:     "https://github.com/owner/repo/issues/123",
			wantErr: true,
		},
		{
			name:    "empty string",
			url:     "",
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			owner, repo, prNum, err := parsePRURL(tt.url)

			if tt.wantErr {
				if err == nil {
					t.Errorf("parsePRURL(%q) expected error, got none", tt.url)
				}
				return
			}

			if err != nil {
				t.Errorf("parsePRURL(%q) unexpected error: %v", tt.url, err)
				return
			}

			if owner != tt.wantOwner {
				t.Errorf("owner = %v, want %v", owner, tt.wantOwner)
			}
			if repo != tt.wantRepo {
				t.Errorf("repo = %v, want %v", repo, tt.wantRepo)
			}
			if prNum != tt.wantPR {
				t.Errorf("prNum = %v, want %v", prNum, tt.wantPR)
			}
		})
	}
}
