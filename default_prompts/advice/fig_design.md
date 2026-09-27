description: Design beautiful figures

Improve the figure so that it communicates its main message clearly, quickly, and professionally.

Use these principles:

- **Clarity first:** The figure should have one obvious purpose. Remove anything that does not help communicate it.
- **Declutter:** Avoid unnecessary grid lines, heavy borders, repeated labels, decorative effects, and other visual noise.
- **Direct labeling:** Label curves, bars, regions, or panels directly when practical instead of forcing the reader to repeatedly consult a legend.
- **Honest axes:** Bar-chart axes should normally start at zero. Clearly mark any axis break, logarithmic scale, or unusual transformation.
- **Consistency:** Use a consistent font family, font sizes, line weights, marker styles, spacing, and visual language throughout the figure and, when relevant, across related figures.
- **Readable typography:** Make labels comfortably readable at the figure's final display size. Avoid tiny text.
- **Alignment and proximity:** Align related elements and panels. Use white space and proximity to make grouping and relationships obvious.
- **Restrained color:** Use color purposefully to distinguish important categories or relationships, not decoratively.
- **Technical quality:** Prefer vector output such as SVG or PDF for the final figure when appropriate. Prefer TikZ.

Importantly, **inspect the actual rendered figure rather than judging only from the source code**. During development, convert or render the figure to a PNG and look at it. Check for crowding, poor alignment, tiny text, confusing hierarchy, excessive whitespace, clipping, and other visual problems. Make adjustments and render it again as needed.

Optimize for what a human reader will actually see.
