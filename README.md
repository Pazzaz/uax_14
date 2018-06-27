# UAX_14

A Rust library to give Unicode aware suggestions of where to insert line breaks in a
text, in accordance to [UAX #14].

It is implemented without any bells and whistles, in full accordance to the annex. It
passes 7282/7312 of the official tests. The tests it doesn't pass are the ones that
expect an implementation of the algorithm from [LB24]. This library uses the more
conservative method from [LB25].

[UAX #14]: https://www.unicode.org/reports/tr14/
[LB24]: https://www.unicode.org/reports/tr14/#LB24
[LB25]: https://www.unicode.org/reports/tr14/#LB25