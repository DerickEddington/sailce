# Sailce

An implementation of the [Willow](https://willowprotocol.org/) protocol for peer-to-peer data
stores.  Currently, Sailce is very incomplete.

The word ["sailce"](https://la-lojban.github.io/sutysisku/lojban/index.html#sisku=sailce) means
willow, in the Lojban language.  Maybe this name should be changed to something less weird, but
it serves as a distinct code-name for now.

Mostly, I just wanted to explore what it's like to make an example application that uses the [Private Area Intersection](https://willowprotocol.org/specs/pai/index.html#private_area_intersection) and the [Encrypting](https://willowprotocol.org/specs/e2e/index.html#e2e).  Also, I wanted to see if the PAI and Encrypting can be integrated with [Iroh](https://github.com/n0-computer/iroh) (which incorporates the Willow Data Model).  I haven't done any of this yet.

But I have made a crate for the [Data Model](./crates/data_model) which is usable.  Attempting the above will need to build on this.

I'm unsure if I'll make much further progress or when and how long it'll take, since I'm trying
to do this in my free time while on sabbatical before I have to find another serfdom job.
