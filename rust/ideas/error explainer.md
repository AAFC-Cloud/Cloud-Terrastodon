Client receives error:

> The client 'john.doe@agr.gc.ca' with object id '55555555-5555-5555-5555-555555555555' does not have authorization to perform action 'Microsoft.Something/register/action' over scope '/subscriptions/55555555-5555-5555-5555-555555555555' or the scope is invalid. If access was recently granted, please refresh your credentials. (Code: AuthorizationFailed)

Root cause analysis:

- The error is related to performing provider registration on a subscription.
- If the client is a member of the cloud team, shrug
- If the client is not a member of the cloud team, suggest they contact their fill out FORM_XYZ


===

If we could create a single text box that ANY error could be shoved in for an explanation, that could be neat. Structural analysis could help give a direct mapping from problem=>solution, but LLMs could give general suggestions.
