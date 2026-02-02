package main

import (
	"flag"
	"fmt"
	"os"
	"regexp"
	"strconv"
	"strings"
)

func parsePRURL(url string) (owner, repo string, prNumber int, err error) {
	url = strings.TrimRight(url, "/")

	// Handle full URLs: https://github.com/OWNER/REPO/pull/123
	if strings.HasPrefix(url, "https://github.com/") {
		parts := strings.Split(strings.TrimPrefix(url, "https://github.com/"), "/")
		if len(parts) >= 4 && parts[2] == "pull" {
			num, err := strconv.Atoi(parts[3])
			if err != nil {
				return "", "", 0, fmt.Errorf("invalid PR number in URL: %s", parts[3])
			}
			return parts[0], parts[1], num, nil
		}
	}

	// Handle shorthand: owner/repo#123
	re := regexp.MustCompile(`^([^/]+)/([^#]+)#(\d+)$`)
	matches := re.FindStringSubmatch(url)
	if len(matches) == 4 {
		num, _ := strconv.Atoi(matches[3])
		return matches[1], matches[2], num, nil
	}

	return "", "", 0, fmt.Errorf("cannot parse PR URL: %s", url)
}

func main() {
	// Input flags
	prArg := flag.String("pr", "", "PR URL (https://github.com/owner/repo/pull/123) or owner/repo#123 format")
	owner := flag.String("owner", "", "Repository owner")
	ownerShort := flag.String("o", "", "Repository owner (shorthand)")
	repo := flag.String("repo", "", "Repository name")
	repoShort := flag.String("r", "", "Repository name (shorthand)")
	prNumber := flag.Int("pr-number", 0, "Pull request number")
	prNumberShort := flag.Int("n", 0, "Pull request number (shorthand)")

	// Filter flags
	author := flag.String("author", "", "Filter comments by author username")
	authorShort := flag.String("a", "", "Filter comments by author username (shorthand)")
	mostRecent := flag.Bool("most-recent", false, "Show only the most recent comment per file")
	mostRecentShort := flag.Bool("m", false, "Show only the most recent comment per file (shorthand)")

	// Output flags
	format := flag.String("format", "claude", "Output format: claude, grouped, flat, minimal, json")
	formatShort := flag.String("f", "", "Output format (shorthand)")
	noSnippet := flag.Bool("no-snippet", false, "Exclude code snippets from output")
	snippetLines := flag.Int("snippet-lines", 15, "Maximum lines in code snippets")
	output := flag.String("output", "", "Write output to file instead of stdout")
	outputShort := flag.String("O", "", "Write output to file (shorthand)")

	flag.Usage = func() {
		fmt.Fprintf(os.Stderr, "pr-comments - Fetch and format GitHub PR comments for LLM consumption\n\n")
		fmt.Fprintf(os.Stderr, "Usage:\n")
		fmt.Fprintf(os.Stderr, "  pr-comments [flags] [PR_URL]\n\n")
		fmt.Fprintf(os.Stderr, "Examples:\n")
		fmt.Fprintf(os.Stderr, "  pr-comments https://github.com/owner/repo/pull/123\n")
		fmt.Fprintf(os.Stderr, "  pr-comments owner/repo#123\n")
		fmt.Fprintf(os.Stderr, "  pr-comments -o owner -r repo -n 123\n\n")
		fmt.Fprintf(os.Stderr, "Flags:\n")
		flag.PrintDefaults()
	}

	flag.Parse()

	// Handle positional argument (PR URL)
	args := flag.Args()
	prURL := *prArg
	if len(args) > 0 && prURL == "" {
		prURL = args[0]
	}

	// Merge short and long flags
	if *ownerShort != "" && *owner == "" {
		*owner = *ownerShort
	}
	if *repoShort != "" && *repo == "" {
		*repo = *repoShort
	}
	if *prNumberShort != 0 && *prNumber == 0 {
		*prNumber = *prNumberShort
	}
	if *authorShort != "" && *author == "" {
		*author = *authorShort
	}
	if *mostRecentShort {
		*mostRecent = true
	}
	if *formatShort != "" && *format == "claude" {
		*format = *formatShort
	}
	if *outputShort != "" && *output == "" {
		*output = *outputShort
	}

	// Determine owner, repo, and PR number
	var finalOwner, finalRepo string
	var finalPRNumber int

	if prURL != "" {
		var err error
		finalOwner, finalRepo, finalPRNumber, err = parsePRURL(prURL)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			os.Exit(1)
		}
	} else if *owner != "" && *repo != "" && *prNumber != 0 {
		finalOwner = *owner
		finalRepo = *repo
		finalPRNumber = *prNumber
	} else {
		flag.Usage()
		fmt.Fprintf(os.Stderr, "\nError: Provide a PR URL or --owner, --repo, and --pr-number\n")
		os.Exit(1)
	}

	// Validate format
	validFormats := map[string]bool{
		"claude":  true,
		"grouped": true,
		"flat":    true,
		"minimal": true,
		"json":    true,
	}
	if !validFormats[*format] {
		fmt.Fprintf(os.Stderr, "Error: Invalid format '%s'. Must be one of: claude, grouped, flat, minimal, json\n", *format)
		os.Exit(1)
	}

	// Fetch comments
	rawComments, err := FetchPRComments(finalOwner, finalRepo, finalPRNumber)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	prInfo, err := FetchPRInfo(finalOwner, finalRepo, finalPRNumber)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	// Parse comments
	comments := ParseComments(rawComments)

	// Apply filters
	if *author != "" {
		comments = FilterByAuthor(comments, *author)
	}

	if *mostRecent {
		comments = GetMostRecentPerFile(comments)
	}

	// Format output
	includeSnippet := !*noSnippet
	var outputStr string

	switch *format {
	case "json":
		outputStr = FormatCommentsJSON(comments, includeSnippet, *snippetLines)
	case "grouped":
		outputStr = FormatCommentsGrouped(comments, includeSnippet, *snippetLines)
	case "flat":
		outputStr = FormatCommentsFlat(comments, includeSnippet, *snippetLines)
	case "minimal":
		outputStr = FormatCommentsMinimal(comments)
	default: // claude
		outputStr = FormatForClaude(comments, prInfo.HTMLURL, prInfo.Title, includeSnippet, *snippetLines)
	}

	// Write output
	if *output != "" {
		err := os.WriteFile(*output, []byte(outputStr), 0644)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error writing to file: %v\n", err)
			os.Exit(1)
		}
		fmt.Fprintf(os.Stderr, "Output written to %s\n", *output)
	} else {
		fmt.Println(outputStr)
	}
}
