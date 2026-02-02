"""Data models for PR comments."""

from dataclasses import dataclass
from datetime import datetime
from typing import Optional


@dataclass
class PRComment:
    """Represents a parsed PR comment with essential information."""

    id: int
    file_path: str
    line_number: Optional[int]
    start_line: Optional[int]
    author: str
    body: str
    created_at: datetime
    updated_at: datetime
    diff_hunk: str
    html_url: str

    def get_code_snippet(self, max_lines: int = 10) -> str:
        """Extract a relevant code snippet from the diff hunk.

        Args:
            max_lines: Maximum number of lines to include in the snippet.

        Returns:
            A trimmed code snippet focusing on the commented area.
        """
        if not self.diff_hunk:
            return ""

        lines = self.diff_hunk.split("\n")

        # Remove the diff header line (starts with @@)
        content_lines = [line for line in lines if not line.startswith("@@")]

        if len(content_lines) <= max_lines:
            return "\n".join(content_lines)

        # Take the last max_lines lines (most relevant to the comment)
        return "\n".join(content_lines[-max_lines:])

    def get_line_info(self) -> str:
        """Get a human-readable line number string."""
        if self.start_line and self.line_number and self.start_line != self.line_number:
            return f"lines {self.start_line}-{self.line_number}"
        elif self.line_number:
            return f"line {self.line_number}"
        else:
            return "line unknown"
