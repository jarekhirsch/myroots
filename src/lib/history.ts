// The most important events in the history of Poland (for the whole country),
// shown in the background of a lifeline. Built in, no network.
import { t } from './i18n/index.svelte';

export interface HistoryEvent {
  /** YYYY, YYYY-MM or YYYY-MM-DD */
  date: string;
  title: () => string;
}

export const POLAND: HistoryEvent[] = [
  { date: '1791-05-03', title: () => t('The Constitution of 3 May is adopted') },
  { date: '1795-10-24', title: () => t('Third partition: Poland disappears from the map') },
  { date: '1807-07-09', title: () => t('The Duchy of Warsaw is created') },
  { date: '1815-06-09', title: () => t('Congress of Vienna: the Kingdom of Poland under the tsar') },
  { date: '1830-11-29', title: () => t('The November Uprising breaks out') },
  { date: '1848-03', title: () => t('Spring of Nations') },
  { date: '1863-01-22', title: () => t('The January Uprising breaks out') },
  { date: '1864-03-02', title: () => t('Peasants in the Kingdom of Poland become owners of their land') },
  { date: '1905-01', title: () => t('Revolution of 1905 in the Kingdom of Poland') },
  { date: '1914-07-28', title: () => t('The First World War breaks out') },
  { date: '1918-10', title: () => t('The Spanish flu pandemic reaches Poland') },
  { date: '1918-11-11', title: () => t('Poland regains independence') },
  { date: '1920-08-15', title: () => t('Battle of Warsaw in the Polish–Soviet War') },
  { date: '1926-05-12', title: () => t('The May Coup') },
  { date: '1929-10', title: () => t('The Great Depression begins') },
  { date: '1939-09-01', title: () => t('Germany invades Poland: the Second World War begins') },
  { date: '1939-09-17', title: () => t('The Soviet Union invades Poland from the east') },
  { date: '1940-02-10', title: () => t('Mass deportations of Poles deep into the Soviet Union begin') },
  { date: '1940-04', title: () => t('The Katyn massacre') },
  { date: '1942-07-22', title: () => t('The Holocaust: deportations from the Warsaw Ghetto to Treblinka') },
  { date: '1943-07-11', title: () => t('Peak of the Volhynian massacre') },
  { date: '1944-08-01', title: () => t('The Warsaw Uprising breaks out') },
  { date: '1945-05-08', title: () => t('End of the Second World War in Europe') },
  { date: '1945-08-02', title: () => t('New borders: resettlement from the east to the western lands') },
  { date: '1947-04-28', title: () => t('Operation Vistula') },
  { date: '1950-10-30', title: () => t('Currency exchange: savings lose most of their value') },
  { date: '1956-06-28', title: () => t('Poznań June protests') },
  { date: '1956-10', title: () => t('The Polish October: the political thaw') },
  { date: '1968-03', title: () => t('March 1968: student protests and the antisemitic campaign') },
  { date: '1970-12-14', title: () => t('December 1970: protests on the Baltic coast') },
  { date: '1976-06-25', title: () => t('June 1976: protests in Radom and Ursus') },
  { date: '1978-10-16', title: () => t('Karol Wojtyła becomes Pope John Paul II') },
  { date: '1980-08-31', title: () => t('August Agreements: Solidarity is born') },
  { date: '1981-12-13', title: () => t('Martial law is declared') },
  { date: '1989-06-04', title: () => t('Partly free elections: the end of communism') },
  { date: '1995-01-01', title: () => t('Redenomination of the złoty') },
  { date: '1997-07', title: () => t('The Millennium Flood') },
  { date: '1999-03-12', title: () => t('Poland joins NATO') },
  { date: '2004-05-01', title: () => t('Poland joins the European Union') },
  { date: '2005-04-02', title: () => t('Death of John Paul II') },
  { date: '2010-04-10', title: () => t('The Smolensk air disaster') },
  { date: '2020-03', title: () => t('The COVID-19 pandemic') },
  { date: '2022-02-24', title: () => t('Russia invades Ukraine: millions of refugees come to Poland') },
];

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
