# Sailce

An experimental exploration of select parts of the [Willow](https://willowprotocol.org/) protocol for
peer-to-peer data stores.  Currently, Sailce is very incomplete.

The word ["sailce"](https://la-lojban.github.io/sutysisku/lojban/index.html#sisku=sailce) means
willow, in the Lojban language.  Maybe this name should be changed to something less weird, but
it serves as a distinct code-name for now.

This started because I wanted to explore what it's like to make a basic example application that
uses the [Private Area
Intersection](https://willowprotocol.org/specs/pai/index.html#private_area_intersection) and the
[Encrypting](https://willowprotocol.org/specs/e2e/index.html#e2e).  Also, I wanted to see if the
PAI and Encrypting can be integrated with [Iroh](https://github.com/n0-computer/iroh) (which
incorporates the Willow Data Model).  I haven't done any of this yet.

But I have made libraries for the [Data Model](./packages/data_model) and the [Path
Encryption](./packages/path_crypto) which are usable and might have some interesting approaches.
Attempting the above will need to build on these.

The design of the APIs involves tradeoffs to support `no_std` usages while also supporting `std`
usages, and involves much generics for greater flexibility that is intended to enable other
libraries to abstract over what implements those aspects while utilizing the core concrete
functionalities that are provided, and is intended to enable an application to integrate multiple
implementations of Willow parts by wrapping their APIs in the generic APIs to unify them.  It
remains to be seen how well the current design achieves these intents.

I'm unsure if I'll make much further progress or when and how long it'll take, since I'm trying
to do this in my free time.
