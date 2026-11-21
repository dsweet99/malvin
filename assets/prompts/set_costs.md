Set up the per-token USD cost rates for {{ model_id }} in ~/.malvinconf/config.toml.

Research the current rates, then edit only that configuration. Do not change repository files.
 - For simplicity, just mark self-hosted models (local, Modal, other servers) as $0/token in all cost fields.

Backends might be configured to use portkey, bedrock, or other services, which might publish
 different rates from the standard provider rates. Please take care to determine
 the actual rates the user will be charged.

Finish after the configuration is saved.
