## Notes on Plonky2 code
1. The code first sets up the basic configuration, from the names of which I can assume it'll use the Goldilocks field and Poseidon for the FRI commitments.
2. Then, it defines the constraints with the multiplication and addition gates for the chosen statement, $x^2 - 4x + 7$. The statement is being deconstructed into smaller pieces during this step.
3. Next comes the registration of input and output values and the creation of a proof with $x$ equal to 1.
4. Finally, the code also verifies the proof.

## The other examples
I've made a codespace to test them.
