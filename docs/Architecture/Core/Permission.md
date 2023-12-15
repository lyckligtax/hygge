# Permission Model
RBAC
# Rolle vs Liste von mitgliedern
jedes Mitglied darf unterschiedliche Dinge in einem Verein
wie discord
es gibt eine normale Mitglieder rolle
es gibt eine adminrolle in welcher der Gründer standardmäßig ist
es können weitere rollen mit unterschiedlichen Rechten angelegt werden

CanDo:
canWriteMail(TenantId) <- tenant_role_admin + tenant_role_Mail
tenant_role_admin <- ProfileId

2 interfaces: eins mit Recht -> Rechtbesitzern & Rechtbesitzer -> Recht ja/nein/vielleicht
