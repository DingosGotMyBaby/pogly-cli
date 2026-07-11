use crate::update;

pub const HELP_TEMPLATE: &str = "{before-help}{usage-heading} {usage}\n\n{all-args}{after-help}";

const LOGO: &str = r#"@@@@@@@.                                            :@@@@@@%
@@@@@@@=--------------------------------------------+@@@@@@%
@@@@@@@*++++++++++++++++++++++++++++++++++++++++++++#@@@@@@@
+++#@++                                             .+*@*++=
   -@                                                 .@:
   =@.                                                :@-
   =@.                                                :@-
   =@.                                                :@-
   =@.            :++=-:.                             :@-
   =@.            :#*****++-:.                        :@-
   =@.             =***********+=-:.                  :@-
   =@.              +****************+=:.             :@-
   =@.              .*********************+=-:.       :@-
   =@.               :**************************+=-.. :@:
   =@.                =******************************++%=.
   =@.                 +***********************************+-
   =@.                 .************************************=
   =@.                  -****************************++%=.
   =@.                   =*********************+=-:.  :@:
   =@.                    ****************=-:.        :@-
   =@.                    .**************:            :@-
   =@.                     -************=             :@-
   =@.                      =***********              :@-
   =@.                       **********:              :@-
   =@.                       :********=               :@:
...=@:.                       -*******               .-@-..
@@@@@@@-.......................******=..............-@@@@@@%
@@@@@@@#********************************************#@@@@@@%
@@@@@@@.                        :**+.               :@@@@@@%
+++++++.                          .                 .+=++=+="#;

pub fn banner() -> String {
    let mut out = String::new();
    if let Some((current, latest)) = update::update_available() {
        out.push_str(&format!(
            "A new version of Pogly CLI is available: v{latest} (current: v{current})\nRun `pogly version upgrade` to update.\n\n"
        ));
    }
    out.push_str(LOGO);
    out.push_str("\n\nPogly Command Line Tool\nEasily interact with a Pogly overlay\n\nGive us feedback in our Discord server:\nhttps://discord.gg/pogly");
    out
}
