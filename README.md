# Simple runuser
runuser, but in rust, and simpler!
this implelemntation of runuser does not use libpam, and instead, requires
the shell or the parents of it to be root, in order for it to be root and be
able to switch users.
