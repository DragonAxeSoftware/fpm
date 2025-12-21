---
applyTo: "**"
---

Lessons learned from past work sessions:

- Implemented blockchain signature fetching using trait-based architecture: separated concerns into OpReturnMessageProvider (chain operations) and SharedDocCertificateCodec (serialization), enabling dependency injection with Arc<dyn Trait> for multi-chain support and testability; moved codec to dedicated sig_codecs module for cross-chain reuse
