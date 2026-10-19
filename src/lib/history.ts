// The most important events in the history of Poland since 1600 (for the whole
// country: wars, partitions, uprisings, reforms, epidemics, emigration), shown in
// the background of a lifeline. Built in, no network; the user can switch any
// of them off and add their own in the settings.
import { t } from './i18n/index.svelte';

export interface HistoryEvent {
  /** YYYY, YYYY-MM or YYYY-MM-DD */
  date: string;
  title: () => string;
}

/** An event shown in the background: built in (id = its date) or added by the user. */
export interface Background {
  id: string;
  date: string;
  title: string;
  own: boolean;
}

export const POLAND: HistoryEvent[] = [
  { date: '1605-09-27', title: () => t("Battle of Kircholm: victory over the Swedes") },
  { date: '1606-08', title: () => t("Zebrzydowski Rebellion of the nobility") },
  { date: '1610-07-04', title: () => t("Battle of Klushino; Polish troops enter Moscow") },
  { date: '1611-06-13', title: () => t("Smolensk is captured after a long siege") },
  { date: '1620-10-07', title: () => t("Defeat at Cecora: Hetman Żółkiewski is killed") },
  { date: '1621-10', title: () => t("Defence of Khotyn against the Ottoman army") },
  { date: '1629-09-26', title: () => t("Truce of Altmark with Sweden") },
  { date: '1648', title: () => t("Khmelnytsky Uprising of the Cossacks") },
  { date: '1652', title: () => t("Plague epidemic across the Commonwealth") },
  { date: '1652-03-09', title: () => t("First use of the liberum veto in the Sejm") },
  { date: '1655-07', title: () => t("The Swedish Deluge begins") },
  { date: '1655-11', title: () => t("Defence of Jasna Góra") },
  { date: '1656-04-01', title: () => t("Lwów Oath of King John Casimir") },
  { date: '1660-05-03', title: () => t("Peace of Oliva ends the war with Sweden") },
  { date: '1667-01-30', title: () => t("Truce of Andrusovo: Kyiv and the left-bank Ukraine are lost") },
  { date: '1672-08', title: () => t("The Ottomans capture Kamieniec Podolski") },
  { date: '1673-11-11', title: () => t("Victory over the Ottomans at Khotyn") },
  { date: '1674-05-21', title: () => t("John III Sobieski is elected king") },
  { date: '1683-09-12', title: () => t("Relief of Vienna by King John III Sobieski") },
  { date: '1686-05-06', title: () => t("Eternal Peace with Russia") },
  { date: '1697-06-27', title: () => t("Augustus II of Saxony is elected king") },
  { date: '1699-01-26', title: () => t("Treaty of Karlowitz: Podolia returns to Poland") },
  { date: '1700-02', title: () => t("The Great Northern War begins") },
  { date: '1708', title: () => t("Great plague epidemic (until 1712)") },
  { date: '1709-07-08', title: () => t("Battle of Poltava; Augustus II returns to the throne") },
  { date: '1717-02-01', title: () => t("The Silent Sejm") },
  { date: '1733', title: () => t("War of the Polish Succession") },
  { date: '1747-08-08', title: () => t("The Załuski Library opens in Warsaw") },
  { date: '1764-09-07', title: () => t("Stanisław August Poniatowski is elected king") },
  { date: '1768-02-29', title: () => t("Bar Confederation against Russian influence") },
  { date: '1770', title: () => t("Plague in the south-east of the Commonwealth") },
  { date: '1772-08-05', title: () => t("First partition of Poland") },
  { date: '1773-10-14', title: () => t("Commission of National Education: the first ministry of education") },
  { date: '1788-10-06', title: () => t("The Great (Four-Year) Sejm begins") },
  { date: '1791-05-03', title: () => t("The Constitution of 3 May is adopted") },
  { date: '1792-05', title: () => t("Targowica Confederation and the war with Russia") },
  { date: '1793-01-23', title: () => t("Second partition of Poland") },
  { date: '1794-03-24', title: () => t("Kościuszko Uprising") },
  { date: '1794-04-04', title: () => t("Battle of Racławice: the scythe-bearers") },
  { date: '1795-10-24', title: () => t("Third partition: Poland disappears from the map") },
  { date: '1797-07', title: () => t("Polish Legions in Italy; Dąbrowski's Mazurka is written") },
  { date: '1806-11', title: () => t("Napoleon's army enters Poland; uprising in Greater Poland") },
  { date: '1807-07-09', title: () => t("The Duchy of Warsaw is created") },
  { date: '1807-07-22', title: () => t("Constitution of the Duchy of Warsaw: peasants become personally free") },
  { date: '1808-05-01', title: () => t("Napoleonic Code: civil registers of births, marriages and deaths begin") },
  { date: '1809-04-19', title: () => t("Battle of Raszyn") },
  { date: '1812-06', title: () => t("Napoleon's campaign against Russia") },
  { date: '1815-06-09', title: () => t("Congress of Vienna: the Kingdom of Poland under the tsar") },
  { date: '1816-11-19', title: () => t("The University of Warsaw is founded") },
  { date: '1830-11-29', title: () => t("The November Uprising breaks out") },
  { date: '1831-05', title: () => t("Cholera epidemic") },
  { date: '1831-09-08', title: () => t("Fall of Warsaw; the Great Emigration begins") },
  { date: '1834-06', title: () => t("Mickiewicz publishes Pan Tadeusz") },
  { date: '1846-02', title: () => t("Kraków Uprising and the Galician peasant revolt") },
  { date: '1847', title: () => t("Famine and typhus in Galicia") },
  { date: '1848-03', title: () => t("Spring of Nations") },
  { date: '1848-04-22', title: () => t("Serfdom is abolished in Galicia") },
  { date: '1848-06', title: () => t("Cholera epidemic") },
  { date: '1852', title: () => t("Cholera epidemic (until 1855)") },
  { date: '1863-01-22', title: () => t("The January Uprising breaks out") },
  { date: '1864-03-02', title: () => t("Peasants in the Kingdom of Poland become owners of their land") },
  { date: '1866', title: () => t("Cholera epidemic") },
  { date: '1867-12', title: () => t("Galicia gains autonomy within Austria-Hungary") },
  { date: '1872', title: () => t("Kulturkampf: Germanisation in the Prussian partition") },
  { date: '1873', title: () => t("Cholera epidemic") },
  { date: '1885-03', title: () => t("Prussian deportations of Poles without German citizenship") },
  { date: '1886-04', title: () => t("Prussian Settlement Commission buys up Polish land") },
  { date: '1890', title: () => t("Mass emigration 'for bread' to America") },
  { date: '1892', title: () => t("Cholera epidemic") },
  { date: '1901-05', title: () => t("School strike in Września against teaching in German") },
  { date: '1903-12-10', title: () => t("Maria Skłodowska-Curie receives the Nobel Prize") },
  { date: '1905-01', title: () => t("Revolution of 1905 in the Kingdom of Poland") },
  { date: '1914-07-28', title: () => t("The First World War breaks out") },
  { date: '1914-08-06', title: () => t("Piłsudski's First Cadre Company marches out") },
  { date: '1915-08-05', title: () => t("The Russians leave Warsaw; mass flight of refugees to the east") },
  { date: '1916-11-05', title: () => t("Act of 5 November: the Kingdom of Poland is announced") },
  { date: '1918-10', title: () => t("The Spanish flu pandemic reaches Poland") },
  { date: '1918-11-01', title: () => t("Battles for Lwów: the Polish–Ukrainian war") },
  { date: '1918-11-11', title: () => t("Poland regains independence") },
  { date: '1918-11-28', title: () => t("Women gain the right to vote") },
  { date: '1918-12-27', title: () => t("Greater Poland Uprising") },
  { date: '1919', title: () => t("Typhus epidemic in the war-torn country") },
  { date: '1919-06-28', title: () => t("Treaty of Versailles") },
  { date: '1919-08', title: () => t("First Silesian Uprising") },
  { date: '1920-02-10', title: () => t("Poland's wedding to the sea in Puck") },
  { date: '1920-08-15', title: () => t("Battle of Warsaw in the Polish–Soviet War") },
  { date: '1921-03-17', title: () => t("The March Constitution") },
  { date: '1921-03-18', title: () => t("Peace of Riga ends the Polish–Soviet War") },
  { date: '1921-05', title: () => t("Third Silesian Uprising") },
  { date: '1922-06-20', title: () => t("Part of Upper Silesia joins Poland") },
  { date: '1922-12-16', title: () => t("President Gabriel Narutowicz is assassinated") },
  { date: '1924-04-28', title: () => t("Grabski's reform: the złoty is introduced") },
  { date: '1926-05-12', title: () => t("The May Coup") },
  { date: '1929-10', title: () => t("The Great Depression begins") },
  { date: '1935-05-12', title: () => t("Death of Józef Piłsudski") },
  { date: '1937', title: () => t("Building of the Central Industrial Region") },
  { date: '1938-10-02', title: () => t("Poland takes Zaolzie from Czechoslovakia") },
  { date: '1939-09-01', title: () => t("Germany invades Poland: the Second World War begins") },
  { date: '1939-09-17', title: () => t("The Soviet Union invades Poland from the east") },
  { date: '1939-09-28', title: () => t("Warsaw capitulates") },
  { date: '1939-10-26', title: () => t("German occupation: the General Government is created") },
  { date: '1940-02-10', title: () => t("Mass deportations of Poles deep into the Soviet Union begin") },
  { date: '1940-04', title: () => t("The Katyn massacre") },
  { date: '1940-06-14', title: () => t("First transport of Polish prisoners to Auschwitz") },
  { date: '1940-11-16', title: () => t("The Warsaw Ghetto is closed off") },
  { date: '1941-06-22', title: () => t("Germany attacks the Soviet Union") },
  { date: '1942-02-14', title: () => t("The Home Army (Armia Krajowa) is formed") },
  { date: '1942-07-22', title: () => t("The Holocaust: deportations from the Warsaw Ghetto to Treblinka") },
  { date: '1942-11-27', title: () => t("Expulsions from the Zamość region") },
  { date: '1943-04-19', title: () => t("Warsaw Ghetto Uprising") },
  { date: '1943-07-04', title: () => t("General Władysław Sikorski dies at Gibraltar") },
  { date: '1943-07-11', title: () => t("Peak of the Volhynian massacre") },
  { date: '1944-05-18', title: () => t("Battle of Monte Cassino") },
  { date: '1944-07-22', title: () => t("The PKWN Manifesto: communist rule begins") },
  { date: '1944-08-01', title: () => t("The Warsaw Uprising breaks out") },
  { date: '1944-10-02', title: () => t("The Warsaw Uprising ends; the city's people are driven out") },
  { date: '1945-01-27', title: () => t("Auschwitz is liberated") },
  { date: '1945-02-11', title: () => t("Yalta Conference decides on Poland's borders") },
  { date: '1945-05-08', title: () => t("End of the Second World War in Europe") },
  { date: '1945-08-02', title: () => t("New borders: resettlement from the east to the western lands") },
  { date: '1946-06-30', title: () => t("The '3 times yes' referendum") },
  { date: '1946-07-04', title: () => t("Kielce pogrom") },
  { date: '1947-01-19', title: () => t("Rigged elections to the Sejm") },
  { date: '1947-04-28', title: () => t("Operation Vistula") },
  { date: '1948-12-15', title: () => t("The communist Polish United Workers' Party is founded") },
  { date: '1950-10-30', title: () => t("Currency exchange: savings lose most of their value") },
  { date: '1952-07-22', title: () => t("Constitution of the Polish People's Republic") },
  { date: '1953-09-25', title: () => t("Cardinal Stefan Wyszyński is arrested") },
  { date: '1956-06-28', title: () => t("Poznań June protests") },
  { date: '1956-10', title: () => t("The Polish October: the political thaw") },
  { date: '1963-07', title: () => t("Smallpox outbreak in Wrocław") },
  { date: '1966-04-14', title: () => t("Millennium of the Baptism of Poland") },
  { date: '1968-03', title: () => t("March 1968: student protests and the antisemitic campaign") },
  { date: '1970-12-14', title: () => t("December 1970: protests on the Baltic coast") },
  { date: '1976-06-25', title: () => t("June 1976: protests in Radom and Ursus") },
  { date: '1978-10-16', title: () => t("Karol Wojtyła becomes Pope John Paul II") },
  { date: '1979-06-02', title: () => t("John Paul II's first pilgrimage to Poland") },
  { date: '1980-08-31', title: () => t("August Agreements: Solidarity is born") },
  { date: '1981-12-13', title: () => t("Martial law is declared") },
  { date: '1983-07-22', title: () => t("Martial law is lifted") },
  { date: '1983-10-05', title: () => t("Lech Wałęsa receives the Nobel Peace Prize") },
  { date: '1984-10-19', title: () => t("Father Jerzy Popiełuszko is murdered") },
  { date: '1986-04-26', title: () => t("Chernobyl disaster: iodine for children") },
  { date: '1989-02-06', title: () => t("Round Table talks begin") },
  { date: '1989-06-04', title: () => t("Partly free elections: the end of communism") },
  { date: '1990-01-01', title: () => t("Balcerowicz Plan: the shift to a market economy") },
  { date: '1990-12-09', title: () => t("Lech Wałęsa is elected president") },
  { date: '1993-09-17', title: () => t("The last Russian troops leave Poland") },
  { date: '1995-01-01', title: () => t("Redenomination of the złoty") },
  { date: '1997-04-02', title: () => t("The present Constitution is adopted") },
  { date: '1997-07', title: () => t("The Millennium Flood") },
  { date: '1999-01-01', title: () => t("Administrative reform: 16 voivodeships and counties") },
  { date: '1999-03-12', title: () => t("Poland joins NATO") },
  { date: '2004-05-01', title: () => t("Poland joins the European Union") },
  { date: '2005-04-02', title: () => t("Death of John Paul II") },
  { date: '2007-12-21', title: () => t("Poland joins the Schengen Area") },
  { date: '2010-04-10', title: () => t("The Smolensk air disaster") },
  { date: '2020-03', title: () => t("The COVID-19 pandemic") },
  { date: '2022-02-24', title: () => t("Russia invades Ukraine: millions of refugees come to Poland") },
  { date: '2024-09-15', title: () => t("Floods in south-western Poland") },
];

/** Built-in events that are on, and the user's own ones, in order of date. */
export function backgroundEvents(h: { off: string[]; own: { id: string; date: string; title: string }[] } | undefined): Background[] {
  const off = new Set(h?.off || []);
  const out: Background[] = POLAND.filter((e) => !off.has(e.date)).map((e) => ({ id: e.date, date: e.date, title: e.title(), own: false }));
  for (const e of h?.own || []) if (e.title && e.date) out.push({ id: e.id, date: e.date, title: e.title, own: true });
  return out;
}

const KEY = 'myroots.history';
export function historyShown(): boolean {
  try {
    return localStorage.getItem(KEY) !== '0';
  } catch {
    return true;
  }
}
export function setHistoryShown(v: boolean) {
  try {
    localStorage.setItem(KEY, v ? '1' : '0');
  } catch {
    /* only for this session */
  }
}
