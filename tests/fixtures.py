"""Test fixtures for PR Comments tests."""

# Sample API response data based on the real GitHub API format
SAMPLE_COMMENT_MINIMAL = {
    "url": "https://api.github.com/repos/ROKT/canal/pulls/comments/123",
    "pull_request_review_id": 456,
    "id": 123,
    "node_id": "PRRC_test123",
    "diff_hunk": "@@ -10,3 +10,5 @@ def hello():\n     print('hello')\n+    print('world')\n+    return True",
    "path": "src/main.py",
    "commit_id": "abc123",
    "original_commit_id": "abc123",
    "user": {
        "login": "reviewer1",
        "id": 12345,
        "type": "User",
    },
    "body": "Consider adding a docstring here.",
    "created_at": "2026-01-30T10:00:00Z",
    "updated_at": "2026-01-30T10:00:00Z",
    "html_url": "https://github.com/ROKT/canal/pull/123#discussion_r123",
    "line": 12,
    "original_line": 12,
    "start_line": None,
    "original_start_line": None,
    "side": "RIGHT",
}

SAMPLE_COMMENT_WITH_RANGE = {
    "url": "https://api.github.com/repos/ROKT/canal/pulls/comments/124",
    "pull_request_review_id": 456,
    "id": 124,
    "node_id": "PRRC_test124",
    "diff_hunk": "@@ -20,10 +20,15 @@ class MyClass:\n     def __init__(self):\n         self.value = 0\n+\n+    def process(self):\n+        result = []\n+        for i in range(10):\n+            result.append(i * 2)\n+        return result",
    "path": "src/processor.py",
    "commit_id": "def456",
    "original_commit_id": "def456",
    "user": {
        "login": "reviewer2",
        "id": 67890,
        "type": "User",
    },
    "body": "This loop could be simplified using a list comprehension:\n```python\nresult = [i * 2 for i in range(10)]\n```",
    "created_at": "2026-01-30T11:00:00Z",
    "updated_at": "2026-01-30T12:00:00Z",
    "html_url": "https://github.com/ROKT/canal/pull/123#discussion_r124",
    "line": 30,
    "original_line": 30,
    "start_line": 25,
    "original_start_line": 25,
    "side": "RIGHT",
}

SAMPLE_COMMENT_BOT = {
    "url": "https://api.github.com/repos/ROKT/canal/pulls/comments/125",
    "pull_request_review_id": 789,
    "id": 125,
    "node_id": "PRRC_test125",
    "diff_hunk": "@@ -100,5 +100,10 @@ def fetch_data():\n     data = api.get('/data')\n+    # Process the response\n+    for item in data:\n+        item['processed'] = True\n+    return data",
    "path": "src/api/client.py",
    "commit_id": "ghi789",
    "original_commit_id": "ghi789",
    "user": {
        "login": "devin-ai-integration[bot]",
        "id": 158243242,
        "type": "Bot",
    },
    "body": """🔴 **Potential issue detected**

The mutation of `item` within the loop modifies the original data structure.

<details>
<summary>Click to expand</summary>

### Root Cause
Modifying dictionary items in place can lead to unexpected side effects.

### Recommendation
Create a copy of the data before processing:
```python
processed_data = [dict(item, processed=True) for item in data]
return processed_data
```

</details>

---
*Was this helpful? React with 👍 or 👎*""",
    "created_at": "2026-01-30T14:00:00Z",
    "updated_at": "2026-01-30T14:00:00Z",
    "html_url": "https://github.com/ROKT/canal/pull/123#discussion_r125",
    "line": 105,
    "original_line": 105,
    "start_line": None,
    "original_start_line": None,
    "side": "RIGHT",
}

SAMPLE_COMMENT_NO_LINE = {
    "url": "https://api.github.com/repos/ROKT/canal/pulls/comments/126",
    "pull_request_review_id": 999,
    "id": 126,
    "node_id": "PRRC_test126",
    "diff_hunk": "@@ -1,3 +1,3 @@ README.md\n-# Old Title\n+# New Title",
    "path": "README.md",
    "commit_id": "jkl012",
    "original_commit_id": "jkl012",
    "user": {
        "login": "reviewer1",
        "id": 12345,
        "type": "User",
    },
    "body": "Should this title match the project name?",
    "created_at": "2026-01-29T09:00:00Z",
    "updated_at": "2026-01-29T09:00:00Z",
    "html_url": "https://github.com/ROKT/canal/pull/123#discussion_r126",
    "line": None,
    "original_line": None,
    "start_line": None,
    "original_start_line": None,
    "side": "RIGHT",
}

SAMPLE_COMMENT_SAME_FILE = {
    "url": "https://api.github.com/repos/ROKT/canal/pulls/comments/127",
    "pull_request_review_id": 456,
    "id": 127,
    "node_id": "PRRC_test127",
    "diff_hunk": "@@ -50,3 +50,8 @@ def hello():\n     print('hello')\n+    # Another function\n+    def goodbye():\n+        print('goodbye')",
    "path": "src/main.py",
    "commit_id": "abc123",
    "original_commit_id": "abc123",
    "user": {
        "login": "reviewer2",
        "id": 67890,
        "type": "User",
    },
    "body": "This function should be at module level, not nested.",
    "created_at": "2026-01-30T15:00:00Z",
    "updated_at": "2026-01-30T15:30:00Z",
    "html_url": "https://github.com/ROKT/canal/pull/123#discussion_r127",
    "line": 55,
    "original_line": 55,
    "start_line": 52,
    "original_start_line": 52,
    "side": "RIGHT",
}

# Collection of all sample comments
SAMPLE_COMMENTS = [
    SAMPLE_COMMENT_MINIMAL,
    SAMPLE_COMMENT_WITH_RANGE,
    SAMPLE_COMMENT_BOT,
    SAMPLE_COMMENT_NO_LINE,
    SAMPLE_COMMENT_SAME_FILE,
]

# Sample PR info response
SAMPLE_PR_INFO = {
    "url": "https://api.github.com/repos/ROKT/canal/pulls/123",
    "id": 1234567,
    "html_url": "https://github.com/ROKT/canal/pull/123",
    "number": 123,
    "state": "open",
    "title": "Add new processing features",
    "user": {
        "login": "developer1",
        "id": 11111,
    },
    "body": "This PR adds new data processing capabilities.",
    "created_at": "2026-01-28T08:00:00Z",
    "updated_at": "2026-01-30T16:00:00Z",
}

# Complex comment matching the user's example
SAMPLE_COMMENT_COMPLEX = {
    "url": "https://api.github.com/repos/ROKT/canal/pulls/comments/2748324069",
    "pull_request_review_id": 3730858266,
    "id": 2748324069,
    "node_id": "PRRC_kwDOE2CVus6j0Bjl",
    "diff_hunk": """@@ -3023,3 +3024,379 @@ def collect_upc_updates(
                sqvav.value = str(origin_supplier_variant.upc)
                sqvavs_to_update.append(sqvav)
    return sqvavs_to_update
+
+
+@dataclass
+class CategoryAttributeLink:
+    \"\"\"Data class for category-attribute link information.\"\"\"
+
+    category: MiraklCategory
+    attribute: MiraklAttribute
+    requirement_level: str
+    is_variant: bool""",
    "path": "canal/apps/mirakl/utils.py",
    "commit_id": "d10b5dc6d82e5f4384cfcac962e2c8358a1dc2a7",
    "original_commit_id": "d10b5dc6d82e5f4384cfcac962e2c8358a1dc2a7",
    "user": {
        "login": "devin-ai-integration[bot]",
        "id": 158243242,
        "type": "Bot",
    },
    "body": """🔴 **QuerySet re-evaluation causes bulk_update to update unmodified objects**

The `_link_value_list_with_attributes` function modifies cache entries in a loop but then passes a re-evaluated queryset to `bulk_update`, causing the modifications to be lost.

**Recommendation:** Convert the queryset to a list before iterating.""",
    "created_at": "2026-01-30T23:06:02Z",
    "updated_at": "2026-01-30T23:06:03Z",
    "html_url": "https://github.com/ROKT/canal/pull/14777#discussion_r2748324069",
    "line": None,
    "original_line": 3376,
    "start_line": None,
    "original_start_line": 3374,
    "side": "RIGHT",
}
