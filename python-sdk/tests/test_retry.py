import unittest

from bochat_sdk.retry import RetryPolicy


class RetryPolicyTests(unittest.TestCase):
    def test_next_delay_grows_and_caps(self):
        policy = RetryPolicy(max_attempts=5, base_delay=0.25, max_delay=0.6)
        self.assertEqual(policy.next_delay(0), 0.25)
        self.assertEqual(policy.next_delay(1), 0.5)
        self.assertEqual(policy.next_delay(2), 0.6)
        self.assertEqual(policy.next_delay(50), 0.6)

    def test_retry_method_boundaries(self):
        self.assertTrue(RetryPolicy.should_retry_method("GET"))
        self.assertTrue(RetryPolicy.should_retry_method("head"))
        self.assertTrue(RetryPolicy.should_retry_method("OPTIONS"))
        self.assertFalse(RetryPolicy.should_retry_method("POST"))
        self.assertFalse(RetryPolicy.should_retry_method("PUT"))
        self.assertFalse(RetryPolicy.should_retry_method("DELETE"))

    def test_retry_status_boundaries(self):
        self.assertTrue(RetryPolicy.should_retry_status(429))
        self.assertTrue(RetryPolicy.should_retry_status(500))
        self.assertTrue(RetryPolicy.should_retry_status(503))
        self.assertFalse(RetryPolicy.should_retry_status(400))
        self.assertFalse(RetryPolicy.should_retry_status(401))
        self.assertFalse(RetryPolicy.should_retry_status(404))


if __name__ == "__main__":
    unittest.main()
