# Summary
I Would like to have a library that provides the access to the properties of the chemical elements of the periodic table. The library should maintain the exhaustive list of the chemical elements, their properties, and their relationships.
The data source should be available at `https://www.iamocean.com/periodic-table/`.
Please maintain it as part of the library in yaml files so that it can be updated easily in the future. The data is presented in the following format. Each of the individual element is kept in the folder `data/elements/` with the name of the element as the file name (e.g., `data/elements/hydrogen.yaml`, `data/elements/helium.yaml`, etc.).
The yaml file for each element should contain the following properties:
- name: Name of the element
- symbol: Symbol of the element
- atomic_number: Atomic number of the element
- atomic_mass: Atomic mass of the element
- group: Group of the element
- period: Period of the element
- block: Block of the element
- electronic_configuration: Electronic configuration of the element
- state: State of the element
- melting_point: Melting point of the element
- boiling_point: Boiling point of the element
- density: Density of the element
- electronegativity: Electronegativity of the element
- oxidation_states: Oxidation states of the element
- isotopes: Isotopes of the element
- discovery_year: Year of discovery of the element
- discoverer: Discoverer of the element
- category: Category of the element

Most of those properties are computed, not part of the yaml. The yaml file should have a minimum set of properties that are not computed but can determine the rest of the properties. The candidate for those properties are:
name, symbol, atomic_number, atomic_mass, discoverer





## Architecture

* Divide the libraries into services, domain, and data layers. 
* The implementation is in Rust, with a workspace project, 3 packages according to the layers
* Each package should have its own unit tests and integration tests.
   

---
## Data Layer
Read the yaml file that contains the properties of the elements.
Load it into the application. This should be done only once when the application is started.

---
## Domain Layer
Perform some calculation, including the electronic configuration based on Aufbau principle,
Hund's rule, and Pauli exclusion principle.
Calculate the atomic mass based on the isotopes and their abundance.
Calculate the electronegativity based on the oxidation states.
Calculate the density based on the atomic mass and atomic radius.
Calculate the melting point based on the atomic mass and atomic radius.
Calculate the boiling point based on the atomic mass and atomic radius.
Calculate the group based on the electronic configuration.
Calculate the period based on the electronic configuration.
Calculate the block based on the electronic configuration.
Calculate the state based on the atomic mass and atomic radius.
Calculate the oxidation states based on the electronic configuration.
Calculate the isotopes based on the atomic mass and atomic radius.
Calculate the discovery_year based on the atomic mass and atomic radius.
Calculate the discoverer based on the atomic mass and atomic radius.
Calculate the category based on the electronic configuration.

---
## Service Layer
Provide interfact to the application and expose the functionalities of the application. The functionality will be, retrieve the detail element by atomic number, atomic mass, name, or symbol.

---
## Implemented Languages
* The application should be implemented using RUST
* Add unit tests for each function.
* Add integration tests for each service.
* Add documentation for each function and service.
