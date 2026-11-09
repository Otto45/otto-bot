// A domain model: every field readonly, so it never changes once made, and no component can change one another holds.
export interface Customer {
  readonly id: string;
  readonly name: string;
  readonly email: string;
}
