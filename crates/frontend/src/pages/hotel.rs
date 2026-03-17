use leptos::prelude::*;

#[component]
pub fn Hotel() -> impl IntoView {
    view! {
        <div class="max-w-5xl mx-auto px-6 py-12">
            <h1 class="font-serif text-4xl text-charcoal mb-4 text-center">"Hotel Accommodations"</h1>
            <div class="flex items-center gap-4 justify-center mb-6 w-48 mx-auto">
                <div class="flex-1 h-px bg-gold/40"></div>
                <svg class="w-2 h-2 fill-gold opacity-60" viewBox="0 0 20 20">
                    <path d="M10 0 L12.5 7.5 L20 10 L12.5 12.5 L10 20 L7.5 12.5 L0 10 L7.5 7.5 Z" />
                </svg>
                <div class="flex-1 h-px bg-gold/40"></div>
            </div>
            <p class="text-center text-charcoal/70 mb-10 max-w-2xl mx-auto">
                "We have reserved room blocks at two nearby downtown Colorado Springs hotels. "
                "Rooms are available Thursday, November 19 through Sunday, November 22, 2026."
            </p>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
                <HotelCard
                    name="SpringHill Suites Downtown Colorado Springs"
                    address="402 S Tejon St, Colorado Springs, CO 80903"
                    rate="$139 / night"
                    rooms=vec![
                        "King Bedroom Studio Suite — 1 King, Trundle bed",
                        "Double Queen Suite — 2 Queen, Trundle bed",
                    ]
                    breakfast="Complimentary"
                    parking="Discounted — $10 / night"
                    deadline="October 20, 2026"
                    contact="Natalia.lora@hotelequities.com"
                    booking_url="https://app.marriott.com/reslink?id=1773335804161&key=GRP&app=resvlink"
                />
                <HotelCard
                    name="Hyatt Place Downtown Colorado Springs"
                    address="201 E Kiowa, Colorado Springs, CO 80903"
                    rate="$129 / night"
                    rooms=vec![
                        "King Bedroom with sofa bed",
                        "2 Queen Beds with sofa bed",
                    ]
                    breakfast="Complimentary continental / buffet-style"
                    parking="Valet — $32 / night; downtown self-park lots also available"
                    deadline="October 19, 2026"
                    contact="mia.pelham@hyatt.com"
                    booking_url="https://www.hyatt.com/shop/denzp?location=Hyatt%20Place%20Colorado%20Springs%20%2F%20Downtown&checkinDate=2026-11-19&checkoutDate=2026-11-22&rooms=1&adults=1&kids=0&corp_id=g-HBWD"
                />
            </div>
            <div class="mt-10 border-t border-gold/30 pt-8 text-center text-sm text-charcoal/60 flex flex-col gap-1">
                <p>"Payment method is kept on file and is not charged at the time of booking."</p>
                <p>"Free cancellation up to 48 hours prior to arrival — no fee."</p>
            </div>
        </div>
    }
}

#[component]
fn HotelCard(
    name: &'static str,
    address: &'static str,
    rate: &'static str,
    rooms: Vec<&'static str>,
    breakfast: &'static str,
    parking: &'static str,
    deadline: &'static str,
    contact: &'static str,
    booking_url: &'static str,
) -> impl IntoView {
    let mailto = format!("mailto:{}", contact);
    view! {
        <div class="border border-gold rounded-lg bg-ivory p-6 flex flex-col gap-5">
            <div>
                <h2 class="font-serif text-xl text-charcoal leading-snug">{name}</h2>
                <p class="font-serif text-3xl text-charcoal mt-1">{rate}</p>
            </div>
            <div class="flex flex-col gap-3 flex-1">
                <DetailRow label="Address" value=address />
                <div>
                    <p class="text-xs font-semibold uppercase tracking-wide text-charcoal/60 mb-1">
                        "Room Options"
                    </p>
                    <ul class="list-disc list-inside text-sm text-charcoal/80 flex flex-col gap-0.5">
                        {rooms.into_iter().map(|r| view! { <li>{r}</li> }).collect_view()}
                    </ul>
                </div>
                <DetailRow label="Breakfast" value=breakfast />
                <DetailRow label="Parking" value=parking />
                <DetailRow label="Book by" value=deadline />
            </div>
            <a
                href=booking_url
                target="_blank"
                rel="noopener noreferrer"
                class="block text-center bg-gold text-ivory font-semibold px-6 py-3 rounded hover:bg-charcoal transition-colors"
            >
                "Book at Group Rate"
            </a>
            <p class="text-xs text-center text-charcoal/50">
                "Need help booking? Email "
                <a href=mailto class="text-sage underline">{contact}</a>
            </p>
        </div>
    }
}

#[component]
fn DetailRow(label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <div>
            <p class="text-xs font-semibold uppercase tracking-wide text-charcoal/60 mb-0.5">{label}</p>
            <p class="text-sm text-charcoal/80">{value}</p>
        </div>
    }
}
