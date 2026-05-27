import sys
import types
import unittest

from bochat_sdk.error import ApiError, HttpStatusError, TransportError

if "httpx" not in sys.modules:
    sys.modules["httpx"] = types.SimpleNamespace(AsyncClient=object)

from bochat_sdk.messages import _should_retry_send_error


class MessageRetryDecisionTests(unittest.TestCase):
    def test_transport_error_is_retryable(self):
        self.assertTrue(_should_retry_send_error(TransportError("network down")))

    def test_api_error_retryable_boundaries(self):
        self.assertTrue(_should_retry_send_error(ApiError("e", "too many", 429)))
        self.assertTrue(_should_retry_send_error(ApiError("e", "server err", 500)))
        self.assertFalse(_should_retry_send_error(ApiError("e", "bad req", 400)))
        self.assertFalse(_should_retry_send_error(ApiError("e", "unauthorized", 401)))

    def test_http_status_error_retryable_boundaries(self):
        self.assertTrue(_should_retry_send_error(HttpStatusError(429, "")))
        self.assertTrue(_should_retry_send_error(HttpStatusError(502, "")))
        self.assertFalse(_should_retry_send_error(HttpStatusError(404, "")))

    def test_unknown_exception_is_not_retryable(self):
        self.assertFalse(_should_retry_send_error(RuntimeError("boom")))


if __name__ == "__main__":
    unittest.main()
