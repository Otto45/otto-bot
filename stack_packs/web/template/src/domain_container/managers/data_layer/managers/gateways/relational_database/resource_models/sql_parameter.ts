// A value bound to a statement's parameter ($1, $2, ...). It names only the types some query binds: add one (e.g.
// `string | number`) with the first query that binds it.
export type SqlParameter = string;
