description: Problems Worth Solving for data visualization

1. **A chart without a clear question.** A figure can contain accurate data yet leave the reader unsure what to compare. State the question the figure should answer, then choose a chart and ordering that make the comparison easy. For example, sort delivery times by route when the question is which routes need attention.

2. **Misleading scales and encodings.** Truncated bar axes, unexplained logarithmic scales, or circles whose radii represent values can distort comparisons. Start bar axes at zero, label transformations, and make symbol area proportional to the value when using size. Check that a value twice as large receives twice the encoded length or area, as appropriate.

3. **Hidden variation, uncertainty, or missing data.** A mean alone leaves the distribution unseen; a line across missing observations conceals the gap. Show distributions, label uncertainty intervals and what they represent, and distinguish missing values from zero. For example, pair group means with individual observations and sample counts.

4. **Labels and colors that are hard to read.** Tiny text, distant legends, and distinctions conveyed only by color make a figure difficult to interpret. Label series directly where practical, include units, and reinforce color with shapes or line styles. Inspect the rendered figure at its intended display size and in grayscale for illegible labels or indistinguishable series.

5. **Visual clutter that obscures the comparison.** Heavy grids, decorative effects, overlapping marks, and too many series compete with the data. Remove decoration, lighten reference lines, and use separate panels or an appropriate summary when marks overlap. For example, show dense observations with a density plot and separate groups into panels that share comparable scales.
