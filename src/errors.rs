error_chain!{
    foreign_links {
        Serenity(::serenity::Error);
    }
}
