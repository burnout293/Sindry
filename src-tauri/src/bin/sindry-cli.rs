//! `sindry.com` : la ligne de commande de Sindry (specification §19). Un
//! programme console — pas d'attribut `windows_subsystem` ici : la console
//! attend sa fin, recoit sa sortie et son code. L'installeur le pose a cote de
//! `sindry.exe` sous le nom `sindry.com`.

fn main() {
    std::process::exit(sindry_lib::ligne_de_commande())
}
