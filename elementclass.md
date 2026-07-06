# Summary

The service should be able to determine if the `class` of an element is `metal`, `non-metal` or  `metalloid`. Need to add the `class` field into `pt-domain` crate. Should be populated into `element.class`. During development, the `class` field can be hardcoded based on the rule-based implementation below. Later, the `class` field should be determined by a function `determine_class(atomicNumber, period, group)`. The `class` field should be available in the `pt-wasm` crate as `ElementClass` enum so that the frontend can use it to determine the color of the element in the periodic table.

## Rule-Based Implementation
Write a data structure a two dimentional array. The row is the `period` and the column is the `group`. The value of the cell is the symbol of the element.
The array should be like this. You need to fill in the blanks.

Period 1: ["H", "He"]
Period 2: ["Li", "Be", "B", "C", "N", "O", "F", "Ne"]
Period 3: ["Na", "Mg", "Al", "Si", "P", "S", "Cl", "Ar"]
Period 4: ["K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As", "Se", "Br", "Kr"]
Period 5: ["Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In", "Sn", "Sb", "Te", "I", "Xe"]
Period 6: ["Cs", "Ba", "La", "Ce", "Pr", "Nd", "Pm", "Sm", "Eu", "Gd", "Tb", "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg", "Tl", "Pb", "Bi", "Po", "At", "Rn"]
Period 7: ["Fr", "Ra", "Ac", "Th", "Pa", "U", "Np", "Pu", "Am", "Cm", "Bk", "Cf", "Es", "Fm", "Md", "No", "Lr", "Rf", "Db", "Sg", "Bh", "Hs", "Mt", "Ds", "Rg", "Cn", "Nh", "Fl", "Mc", "Lv", "Ts", "Og"]


### Definitions
The `class` is defined as below.
Based on the periodic table, the `class` is defined as below.
Since the array's index is 0 based, you need to adjust the index to be 1 based when you use it. For example, if you want to check the class of the element at period 1 and group 1, you need to use array[0][0] .

Iterate over the data. For each element, determine the `class`. For the given data, we already know the element period and group. We only need to determine the `class` using the atomic number (z).

The pseudocode to determine the `class` of an element is as below.
// 1. Absolute Exceptions
  if (z === 1) return "non-metal"; // Hydrogen sits in Group 1 but is a non-metal
  
  // 2. Clear Non-Metal Blocks (Halogens & Noble Gases)
  if (group === 17 || group === 18) return "non-metal";
  
  // 3. Clear Metal Blocks (Lanthanides & Actinides pulled out at the bottom)
  if ((z >= 57 && z <= 71) || (z >= 89 && z <= 103)) return "metal";
  
  // 4. The Metalloid "Staircase" Intersection
  // This looks up the exact 7 elements making the diagonal divider
  const metalloids = new Set([5, 14, 32, 33, 51, 52, 84]);
  if (metalloids.has(z)) return "metalloid";
  
  // 5. Non-Metals tucked neatly above the Metalloid Staircase
  if (period === 2 && group >= 14 && group <= 16) return "non-metal"; // C, N, O
  if (period === 3 && group >= 15 && group <= 16) return "non-metal"; // P, S
  if (period === 4 && group === 16) return "non-metal";               // Se

  // 6. Default Fallback
  // Everything else on the canvas automatically evaluates to a metal
  return "metal";

Expose the function `determine_class(atomicNumber, period, group)` in a service or utility file. 

The result of `determine_class(atomicNumber, period, group)` should be one of the following values:
- "metal"
- "non-metal"
- "metalloid"



### Example

Given an element with the following properties:

atomicNumber = 12
period = 3
group = 2

The function should return "metal".
