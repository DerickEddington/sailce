# sailce_data_model

An implementation of Willow's [Data Model](https://willowprotocol.org/specs/data-model/index.html).

Willow is a system for giving meaningful hierarchical names to arbitrary sequences of bytes
(called _payloads_), not unlike a filesystem, for multiple peers.

The design involves much generics for greater flexibility that is intended to enable other
libraries to work with the Data Model while abstracting over what implements it concretely, and is
intended to enable an application to integrate multiple implementations of Willow parts by
wrapping their APIs in the generic API of this `sailce_data_model` crate to unify them.  It
remains to be seen how well the current design achieves these intents.

TODO:  More test coverage.  (There are some basic tests so far.)
