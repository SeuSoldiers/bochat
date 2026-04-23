from __future__ import annotations

from dataclasses import dataclass


@dataclass
class RetryPolicy:
    max_attempts: int = 3
    base_delay: float = 0.2
    max_delay: float = 2.0

    def next_delay(self, attempt: int) -> float:
        exp = 1 << min(attempt, 8)
        delay = self.base_delay * exp
        return min(delay, self.max_delay)

    @staticmethod
    def should_retry_method(method: str) -> bool:
        return method.upper() in {"GET", "HEAD", "OPTIONS"}

    @staticmethod
    def should_retry_status(status: int) -> bool:
        return status >= 500 or status == 429
