# Tempo.rs

`tempo.rs` is a Rust client library allowing access to RTE's API for EDF's "Tempo" contract option.

## Context
 [EDF][1] is a French state-owned electricity utility company in charge of selling energy to homes and businesses.
 [RTE][2] is a subsidiary of EDF in charge of the operation, maintenance and development of the French high-voltage transmission system.

 EDF's _Tarif Bleu_ contract comes with an option called "Tempo", with which electricity price fluctuates depending on days of the year 
 and time of day.
 There are three types of day:
  - Blue : cheapest prices
  - White : average prices
  - Red : prices can be five times higher than average

 Each day is split in two periods:
  - Peak hours: from 6AM to 10PM
  - Off-peak hours: from 10PM to 6AM

 For example, at the time of writing :
  - during a Blue day, off-peak hour price is 12.96 ct€/kWh
  - during a White day, off-peak hour price is 14.86 ct€/kWh
  - during a Red day, peak hour price is 75.62 ct€/kWh

 See [the official prices table][3] for all prices.

 Given the high differences in prices between periods and days, it can be interesting to have advance knowledge about future days' color.
 As a matter of fact, RTE, who runs the [complex algorith][6] to choose a day's color, is contractually bound to publish next-day color at 10:30AM each day.

 Subsenquently, RTE also publishes a REST API (which this crate is about) allowing to request:
  - historical data (past days' color)
  - next day color

## Usage

### Authentication & Authorization
API usage is authorized through OAuth2. To obtain a client id and client secret, you need to create an account at RTE's [Data Portal][4] and subscribe to the ["Tempo-like supply contract" API][5]. You will be supplied with a file containing a base64-encoded line of text. Keep it securely as it contains a client id and client secret to use in the OAuth2 flow.

OAuth2 credentials can be supplied to `tempo.rs`by two means:
 - `tempo_rs::authorize_with_file`, which will read directly from the file supplied by RTE
 - `tempo_rs::authorize` if you wish to supply id and secret by hand

### Use cases

Once holding a `Tempo` object, there are two use cases.

#### Retrieving historical data

Method `Tempo::calendars` allows to manipulate all the parameters exposed by the underlying REST API: `start_date`, `end_date`, `fallback`. Its main use is to request historical data.
Even though `start_date` and `end_date` are UTC based, official documentation mentions that dates can be supplied from any timezone.
Thus it is not clear what is the effect of the time component.

The two valid ways to call this function are:
 - `start_date` and `end_date` both containing `Some` date/time. In this case, historical data is returned for the period.
 - `None` of `start_date` and `end_date` contain a date/time. In this case, next-day data is returned. See [`Self::next_day()`].
    ///
Official documentation **does not recommand** to request more than 366 days at a time.
Earliest possible date is 09/01/2014.
It is not clear what `fallback` is or what it is used for. Official doc refers to a *degraded mode*.

#### Retrieving next-day data

Method `Tempo::next_day` can be used to request only (and specifically) next-day data. It basically is a short-hand for [`Tempo::calendars()`] with all parameters set to `None`.

## The `TempoCalendars` object 

The `TempoCalendars` object and underlying structure directly mirrors the way the API outputs its results.
Days' color is nested under two layers of object:
1) First it is necessary to descend into the `tempo_like_calendars` field and iterate over `Calendar` objects.
2) Then, iterate over the `values` containing `CalendarValue` objects.

Look at methods `unwrap_first_day_value()` and `unwrap_days_values()` to easily unwrap these two layers.

## The `tempo` binary

This library comes with a `tempo` binary. 
This binary aims to showcase how the library can be used.
While it serves as an example, it also retrieve useful info: it prints on the command line current week colors followed by next-day color.

# DISCLAIMER

This software is provided as-is, without any warranty. I am not in any way affiliated with RTE or EDF or any of their subsdiaries, affiliates or contractors. The data accessed through the API with this crate is property of RTE or affiliates and I am not responsible of any damage or cost incurred by the use of that crate and the data it provides access to. 

 [1]: https://en.wikipedia.org/wiki/%C3%89lectricit%C3%A9_de_France
 [2]: https://en.wikipedia.org/wiki/R%C3%A9seau_de_Transport_d%27%C3%89lectricit%C3%A9
 [3]: https://particulier.edf.fr/content/dam/2-Actifs/Documents/Offres/Grille_prix_Tarif_Bleu.pdf
 [4]: https://data.rte-france.com/
 [5]: https://data.rte-france.com/catalog/-/api/consumption/Tempo-Like-Supply-Contract/v1.1
 [6]: https://www.services-rte.com/files/live/sites/services-rte/files/pdf/20160106_Methode_de_choix_des_jours_Tempo.pdf