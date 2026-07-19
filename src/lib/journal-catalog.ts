// 预置期刊目录：数据手工整理自项目根目录 RSS_List.md，
// 仅收录 Status 为 Verified（含 Verified (New)）且 RSS URL 可用的期刊。
// RSS_List.md 更新后需同步维护本文件。
export interface CatalogJournal {
  name: string;        // 期刊名称
  category: string;    // 学科分类
  rssUrl: string;      // RSS URL
  issn: string;        // ISSN
}

export const CATEGORIES = [
  "Accounting",
  "Economics",
  "Entrepreneurship",
  "Ethics & Social Responsibility",
  "Finance",
  "General Management",
  "Information Systems",
  "Innovation & Technology Management",
  "International Business",
  "Marketing",
  "Operations & Manufacturing",
  "Organizational Behaviour & Human Resources",
  "Practice & General Business",
] as const;

export const JOURNAL_CATALOG: CatalogJournal[] = [
  // Accounting
  { name: "Accounting, Organizations and Society", category: "Accounting", rssUrl: "https://rss.sciencedirect.com/publication/science/03613682", issn: "0361-3682" },
  { name: "Contemporary Accounting Research", category: "Accounting", rssUrl: "https://onlinelibrary.wiley.com/feed/19113846/most-recent", issn: "0823-9150" },
  { name: "Journal of Accounting and Economics", category: "Accounting", rssUrl: "https://rss.sciencedirect.com/publication/science/01654101", issn: "0165-4101" },
  { name: "Journal of Accounting Research", category: "Accounting", rssUrl: "https://onlinelibrary.wiley.com/feed/1475679x/most-recent", issn: "0021-8456" },
  { name: "Review of Accounting Studies", category: "Accounting", rssUrl: "https://link.springer.com/search.rss?facet-journal-id=11142", issn: "1380-6653" },
  // Economics
  { name: "American Economic Review", category: "Economics", rssUrl: "https://pubs.aeaweb.org/action/showFeed?type=etoc&feed=rss&jc=aer", issn: "0002-8282" },
  { name: "Econometrica", category: "Economics", rssUrl: "https://onlinelibrary.wiley.com/feed/14680262/most-recent", issn: "0012-9682" },
  { name: "Journal of Political Economy", category: "Economics", rssUrl: "https://www.journals.uchicago.edu/action/showFeed?type=etoc&feed=rss&jc=jpe", issn: "0022-3808" },
  // Entrepreneurship
  { name: "Journal of Business Venturing", category: "Entrepreneurship", rssUrl: "https://rss.sciencedirect.com/publication/science/08839026", issn: "0883-9026" },
  { name: "Strategic Entrepreneurship Journal", category: "Entrepreneurship", rssUrl: "https://onlinelibrary.wiley.com/feed/1932443x/most-recent", issn: "1932-4391" },
  // Ethics & Social Responsibility
  { name: "Journal of Business Ethics", category: "Ethics & Social Responsibility", rssUrl: "https://link.springer.com/search.rss?facet-journal-id=10551", issn: "0167-4544" },
  // Finance
  { name: "Journal of Finance", category: "Finance", rssUrl: "https://onlinelibrary.wiley.com/feed/15406261/most-recent", issn: "0022-1082" },
  { name: "Journal of Financial and Quantitative Analysis", category: "Finance", rssUrl: "https://www.cambridge.org/core/rss/product/id/E4D6DDA4FE1BF4A6A6F6FAB74789E49C", issn: "0022-1090" },
  { name: "Journal of Financial Economics", category: "Finance", rssUrl: "https://rss.sciencedirect.com/publication/science/0304405X", issn: "0304-405X" },
  // General Management
  { name: "Academy of Management Journal", category: "General Management", rssUrl: "https://journals.aom.org/action/showFeed?type=etoc&feed=rss&jc=amj", issn: "0001-4273" },
  { name: "Academy of Management Review", category: "General Management", rssUrl: "https://journals.aom.org/action/showFeed?type=etoc&feed=rss&jc=amr", issn: "0363-7425" },
  { name: "Journal of Management Studies", category: "General Management", rssUrl: "https://onlinelibrary.wiley.com/feed/14676486/most-recent", issn: "0022-2380" },
  { name: "Management Science", category: "General Management", rssUrl: "https://pubsonline.informs.org/action/showFeed?type=etoc&feed=rss&jc=mnsc", issn: "0025-1909" },
  { name: "Organization Science", category: "General Management", rssUrl: "https://pubsonline.informs.org/action/showFeed?type=etoc&feed=rss&jc=orsc", issn: "1047-7039" },
  { name: "Strategic Management Journal", category: "General Management", rssUrl: "https://onlinelibrary.wiley.com/feed/10970266/most-recent", issn: "0143-2095" },
  // Information Systems
  { name: "Information Systems Research", category: "Information Systems", rssUrl: "https://pubsonline.informs.org/action/showFeed?type=etoc&feed=rss&jc=isre", issn: "1047-7047" },
  { name: "Journal of Management Information Systems", category: "Information Systems", rssUrl: "https://www.jmis-web.org/jmis.xml", issn: "0742-1222" },
  // Innovation & Technology Management
  { name: "Research Policy", category: "Innovation & Technology Management", rssUrl: "https://rss.sciencedirect.com/publication/science/00487333", issn: "0048-7333" },
  // International Business
  { name: "Journal of International Business Studies", category: "International Business", rssUrl: "https://link.springer.com/search.rss?facet-journal-id=41267", issn: "0047-2506" },
  // Marketing
  { name: "Journal of Consumer Psychology", category: "Marketing", rssUrl: "https://myscp.onlinelibrary.wiley.com/action/showFeed?type=etoc&feed=rss&jc=15327663", issn: "1057-7408" },
  { name: "Journal of Consumer Research", category: "Marketing", rssUrl: "https://consumerresearcher.com/feed", issn: "0093-5301" },
  { name: "Journal of the Academy of Marketing Science", category: "Marketing", rssUrl: "https://link.springer.com/search.rss?facet-journal-id=11747", issn: "0092-0703" },
  { name: "Marketing Science", category: "Marketing", rssUrl: "https://pubsonline.informs.org/action/showFeed?type=etoc&feed=rss&jc=mksc", issn: "0732-2399" },
  // Operations & Manufacturing
  { name: "Journal of Operations Management", category: "Operations & Manufacturing", rssUrl: "https://onlinelibrary.wiley.com/feed/18731317/most-recent", issn: "1873-1317" },
  { name: "Manufacturing and Service Operations Management", category: "Operations & Manufacturing", rssUrl: "https://pubsonline.informs.org/action/showFeed?type=etoc&feed=rss&jc=msom", issn: "1523-4614" },
  { name: "Operations Research", category: "Operations & Manufacturing", rssUrl: "https://pubsonline.informs.org/action/showFeed?type=etoc&feed=rss&jc=opre", issn: "0030-364X" },
  // Organizational Behaviour & Human Resources
  { name: "Human Resource Management", category: "Organizational Behaviour & Human Resources", rssUrl: "https://onlinelibrary.wiley.com/feed/17488583/most-recent", issn: "0954-5395" },
  { name: "Journal of Applied Psychology", category: "Organizational Behaviour & Human Resources", rssUrl: "https://psycnet.apa.org/journals/apl.rss", issn: "0021-9010" },
  { name: "Organizational Behavior and Human Decision Processes", category: "Organizational Behaviour & Human Resources", rssUrl: "https://rss.sciencedirect.com/publication/science/07495978", issn: "0749-5978" },
  // Practice & General Business
  { name: "Harvard Business Review", category: "Practice & General Business", rssUrl: "http://feeds.hbr.org/harvardbusiness", issn: "0017-8012" },
  { name: "Sloan Management Review", category: "Practice & General Business", rssUrl: "http://feeds.feedburner.com/mitsmr", issn: "1532-9194" },
];
