package main

// Sample API response data based on the real GitHub API format

var SampleCommentMinimal = RawComment{
	ID:                123,
	Path:              "src/main.py",
	Line:              intPtr(12),
	OriginalLine:      intPtr(12),
	StartLine:         nil,
	OriginalStartLine: nil,
	Body:              "Consider adding a docstring here.",
	CreatedAt:         "2026-01-30T10:00:00Z",
	UpdatedAt:         "2026-01-30T10:00:00Z",
	DiffHunk:          "@@ -10,3 +10,5 @@ def hello():\n     print('hello')\n+    print('world')\n+    return True",
	HTMLURL:           "https://github.com/ROKT/canal/pull/123#discussion_r123",
	User:              struct{ Login string `json:"login"` }{Login: "reviewer1"},
}

var SampleCommentWithRange = RawComment{
	ID:                124,
	Path:              "src/processor.py",
	Line:              intPtr(30),
	OriginalLine:      intPtr(30),
	StartLine:         intPtr(25),
	OriginalStartLine: intPtr(25),
	Body:              "This loop could be simplified using a list comprehension:\n```python\nresult = [i * 2 for i in range(10)]\n```",
	CreatedAt:         "2026-01-30T11:00:00Z",
	UpdatedAt:         "2026-01-30T12:00:00Z",
	DiffHunk:          "@@ -20,10 +20,15 @@ class MyClass:\n     def __init__(self):\n         self.value = 0\n+\n+    def process(self):\n+        result = []\n+        for i in range(10):\n+            result.append(i * 2)\n+        return result",
	HTMLURL:           "https://github.com/ROKT/canal/pull/123#discussion_r124",
	User:              struct{ Login string `json:"login"` }{Login: "reviewer2"},
}

var SampleCommentBot = RawComment{
	ID:                125,
	Path:              "src/api/client.py",
	Line:              intPtr(105),
	OriginalLine:      intPtr(105),
	StartLine:         nil,
	OriginalStartLine: nil,
	Body:              "**Potential issue detected**\n\nThe mutation of `item` within the loop modifies the original data structure.",
	CreatedAt:         "2026-01-30T14:00:00Z",
	UpdatedAt:         "2026-01-30T14:00:00Z",
	DiffHunk:          "@@ -100,5 +100,10 @@ def fetch_data():\n     data = api.get('/data')\n+    # Process the response\n+    for item in data:\n+        item['processed'] = True\n+    return data",
	HTMLURL:           "https://github.com/ROKT/canal/pull/123#discussion_r125",
	User:              struct{ Login string `json:"login"` }{Login: "devin-ai-integration[bot]"},
}

var SampleCommentNoLine = RawComment{
	ID:                126,
	Path:              "README.md",
	Line:              nil,
	OriginalLine:      nil,
	StartLine:         nil,
	OriginalStartLine: nil,
	Body:              "Should this title match the project name?",
	CreatedAt:         "2026-01-29T09:00:00Z",
	UpdatedAt:         "2026-01-29T09:00:00Z",
	DiffHunk:          "@@ -1,3 +1,3 @@ README.md\n-# Old Title\n+# New Title",
	HTMLURL:           "https://github.com/ROKT/canal/pull/123#discussion_r126",
	User:              struct{ Login string `json:"login"` }{Login: "reviewer1"},
}

var SampleCommentSameFile = RawComment{
	ID:                127,
	Path:              "src/main.py",
	Line:              intPtr(55),
	OriginalLine:      intPtr(55),
	StartLine:         intPtr(52),
	OriginalStartLine: intPtr(52),
	Body:              "This function should be at module level, not nested.",
	CreatedAt:         "2026-01-30T15:00:00Z",
	UpdatedAt:         "2026-01-30T15:30:00Z",
	DiffHunk:          "@@ -50,3 +50,8 @@ def hello():\n     print('hello')\n+    # Another function\n+    def goodbye():\n+        print('goodbye')",
	HTMLURL:           "https://github.com/ROKT/canal/pull/123#discussion_r127",
	User:              struct{ Login string `json:"login"` }{Login: "reviewer2"},
}

// Comment where line is nil but original_line has value
var SampleCommentComplex = RawComment{
	ID:                2748324069,
	Path:              "canal/apps/mirakl/utils.py",
	Line:              nil,
	OriginalLine:      intPtr(3376),
	StartLine:         nil,
	OriginalStartLine: intPtr(3374),
	Body:              "QuerySet re-evaluation causes bulk_update to update unmodified objects",
	CreatedAt:         "2026-01-30T23:06:02Z",
	UpdatedAt:         "2026-01-30T23:06:03Z",
	DiffHunk:          "@@ -3023,3 +3024,379 @@ def collect_upc_updates(\n                sqvav.value = str(origin_supplier_variant.upc)\n                sqvavs_to_update.append(sqvav)\n    return sqvavs_to_update",
	HTMLURL:           "https://github.com/ROKT/canal/pull/14777#discussion_r2748324069",
	User:              struct{ Login string `json:"login"` }{Login: "devin-ai-integration[bot]"},
}

var SampleComments = []RawComment{
	SampleCommentMinimal,
	SampleCommentWithRange,
	SampleCommentBot,
	SampleCommentNoLine,
	SampleCommentSameFile,
}

var SamplePRInfo = PRInfo{
	HTMLURL: "https://github.com/ROKT/canal/pull/123",
	Title:   "Add new processing features",
}

func intPtr(i int) *int {
	return &i
}
