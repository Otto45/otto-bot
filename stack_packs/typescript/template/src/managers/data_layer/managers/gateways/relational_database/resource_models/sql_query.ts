import type { SqlParameter } from "#src/managers/data_layer/managers/gateways/relational_database/resource_models/sql_parameter.ts";

// A statement's text when the compiler can see it was written in the code: its type is the literal itself, so a
// statement built at run time (a template with a value in it, a concatenation) is typed `string` and refused.
type LiteralStatement<Statement extends string> = string extends Statement ? never : Statement;

// Values travel as bound parameters, never formatted into the statement, and SqlQuery.of only takes a statement written
// in the code (a string or a template with nothing interpolated), so one built from a value doesn't compile.
export class SqlQuery {
  public readonly statement: string;
  public readonly parameters: readonly SqlParameter[];

  private constructor(statement: string, parameters: readonly SqlParameter[]) {
    this.statement = statement;
    this.parameters = parameters;
  }

  public static of<Statement extends string>(
    statement: LiteralStatement<Statement>,
    parameters: readonly SqlParameter[],
  ): SqlQuery {
    return new SqlQuery(statement, parameters);
  }
}
