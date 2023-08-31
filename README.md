# Amazon KRDS for Serde

Serializer and deserializer implementation for Amazon's KRDS
format (used by Kindle e-readers to store user reading data.)

Warning, some types are fragile, for example Tuple Structs cannot
contain optionals anywhere except at the end. They will fail to
deserialize if the optional is none if this rule is not followed.
More stable implementations may be created as needs arise and I
understand serde more.

Check my other project, [kindle_formats-rs](https://github.com/willemml/kindle_formats-rs) for Rust struct
representations of Kindle data files that use the KRDS format.

Information on how the data is structured in the binary format was
determined by jhowell in [this mobilereads.com thread](https://www.mobileread.com/forums/showthread.php?t=322172&highlight=krds).
