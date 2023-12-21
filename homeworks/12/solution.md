## Videos
They were quite interesting!

## Arithmetic circuit
1. The O3 represents $`(w + 1) * (w + 3) = w^2 + 4w + 3`$
2. If $`w^2 + 4w + 3 = 24`$, then $`w`$ is 3 or -7
3. The constraints:
   - G1: w + 1 - O1 = 0
   - G2: w + 3 - O2 = 0
   - G3: O1 * O2 - O3 = 0
4. With selectors:
   - G1: S1 * (w + 1) + (1 - S1) * w - O1 = 0
   - G2: S2 * (w + 3) + (1 - S2) * 3w - O1 = 0
   - G3: S3 * (O1 + O2) + (1 - S3) * O1 * O2 - O3 = 0