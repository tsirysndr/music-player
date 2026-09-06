import styled from "@emotion/styled";
import { useEffect, useRef, useState } from "react";
import ControlBar from "../../Components/ControlBar";
import Sidebar from "../../Components/Sidebar/SidebarWithData";
import { fetcher } from "../../Api/fetcher";
import { Activity, Disc, Globe, Headphones, Mic, Music, Radio, Search as SearchIcon, Smile, Speaker, Zap } from "@styled-icons/feather";
import { Tabs, Tab } from "baseui/tabs-motion";
import ContentLoader from "react-content-loader";
import { useSearchParams } from "react-router-dom";

type Station = { id:string; name:string; streamUrl:string; source:string; genre:string; country:string; logo:string; bitrate:number };
const categories = [
  {label:"Synthwave",term:"synthwave",Icon:Activity,color:"#ff4fa3"},{label:"Lo-fi",term:"lofi",Icon:Headphones,color:"#39d9e8"},{label:"Jazz",term:"jazz",Icon:Music,color:"#f2c94c"},{label:"Techno",term:"techno",Icon:Speaker,color:"#9b6cff"},
  {label:"Ambient",term:"ambient",Icon:Activity,color:"#39d9e8"},{label:"Classical",term:"classical",Icon:Music,color:"#f052d4"},{label:"Rock",term:"rock",Icon:Zap,color:"#ff4fa3"},{label:"Pop",term:"pop",Icon:Mic,color:"#f052d4"},
  {label:"Electronic",term:"electronic",Icon:Disc,color:"#4d8dff"},{label:"Hip-Hop",term:"hip hop",Icon:Disc,color:"#f2c94c"},{label:"Chillout",term:"chill",Icon:Smile,color:"#39d9e8"},{label:"Dance",term:"dance",Icon:Speaker,color:"#ff4fa3"},
  {label:"Reggae",term:"reggae",Icon:Music,color:"#f2c94c"},{label:"Metal",term:"metal",Icon:Zap,color:"#9b6cff"},{label:"News",term:"news",Icon:Radio,color:"#39d9e8"},{label:"World",term:"world",Icon:Globe,color:"#4d8dff"},
  {label:"House",term:"house",Icon:Speaker,color:"#39d9e8"},{label:"Trance",term:"trance",Icon:Activity,color:"#9b6cff"},{label:"Drum & Bass",term:"drum and bass",Icon:Speaker,color:"#4d8dff"},{label:"Disco",term:"disco",Icon:Disc,color:"#ff4fa3"},
  {label:"Funk",term:"funk",Icon:Disc,color:"#f052d4"},{label:"Soul",term:"soul",Icon:Music,color:"#ff4fa3"},{label:"R&B",term:"r&b",Icon:Mic,color:"#9b6cff"},{label:"Blues",term:"blues",Icon:Music,color:"#4d8dff"},
  {label:"Country",term:"country",Icon:Music,color:"#f2c94c"},{label:"Folk",term:"folk",Icon:Music,color:"#39d9e8"},{label:"Punk",term:"punk",Icon:Zap,color:"#ff4fa3"},{label:"Indie",term:"indie",Icon:Headphones,color:"#f052d4"},
  {label:"Latin",term:"latin",Icon:Music,color:"#f052d4"},{label:"K-Pop",term:"k-pop",Icon:Mic,color:"#ff4fa3"},{label:"Gospel",term:"gospel",Icon:Music,color:"#f2c94c"},{label:"Oldies",term:"oldies",Icon:Disc,color:"#39d9e8"},
  {label:"Soundtrack",term:"soundtrack",Icon:Headphones,color:"#4d8dff"},
];
const Shell=styled.div`display:flex;background:${p=>p.theme.colors.background};min-height:100vh;font-family:RockfordSansRegular;& button,& input{font-family:inherit;}`;
const Main=styled.main`flex:1;min-width:0;`;
const Body=styled.div`padding:24px 32px 130px;`;
const Grid=styled.div`display:grid;grid-template-columns:repeat(4,minmax(120px,1fr));gap:10px;margin-bottom:20px;`;
const Category=styled.button`height:62px;border:1px solid rgba(255,255,255,.08);border-radius:8px;cursor:pointer;color:${p=>p.theme.colors.text};background:${p=>p.theme.colors.secondaryBackground};display:flex;align-items:center;gap:12px;padding:0 14px;text-align:left;&:hover{border-color:#39d9e899;}`;
const CategoryIcon=styled.span<{color:string}>`width:34px;height:34px;flex:0 0 34px;border-radius:8px;display:flex;align-items:center;justify-content:center;background:rgba(255,255,255,.05);color:${p=>p.color};`;
const Row=styled.div`display:flex;align-items:center;gap:14px;padding:10px;border-bottom:1px solid rgba(170,170,170,.13);color:${p=>p.theme.colors.text};`;
const Logo=styled.img`width:44px;height:44px;object-fit:cover;border-radius:6px;`;
const Info=styled.div`flex:1;min-width:0;overflow:hidden;`; const Name=styled.div`font-family:RockfordSansBold;font-weight:700;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;`; const Meta=styled.div`opacity:.6;font-size:12px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;`;
const Action=styled.button`border:0;background:transparent;color:#ab28fc;cursor:pointer;font-size:18px;`;
const Modal=styled.div`position:fixed;inset:0;background:#000a;z-index:20;display:flex;justify-content:center;padding-top:80px;`;
const Panel=styled.div`width:min(620px,calc(100vw - 80px));height:min(520px,calc(100vh - 160px));background:${p=>p.theme.colors.secondaryBackground};border-radius:14px;padding:18px;overflow:auto;`;
const SearchField=styled.div`display:flex;align-items:center;gap:12px;padding:4px 2px 14px;margin-bottom:8px;border-bottom:1px solid rgba(170,170,170,.28);color:${p=>p.theme.colors.secondaryText};`;
const SearchLauncher=styled.button`width:100%;display:flex;align-items:center;gap:12px;margin:14px 0 20px;padding:12px 2px;border:0;border-bottom:1px solid rgba(170,170,170,.28);background:transparent;color:${p=>p.theme.colors.secondaryText};font-size:16px;text-align:left;cursor:text;`;
const Input=styled.input`flex:1;min-width:0;padding:8px 0;border:0;outline:0;background:transparent;color:${p=>p.theme.colors.text};font-size:17px;`;
const CategoryHeader=styled.div`display:flex;align-items:center;gap:14px;margin:10px 0 20px;color:${p=>p.theme.colors.text};`;
const BackButton=styled.button`border:0;background:transparent;color:${p=>p.theme.colors.text};font-size:28px;line-height:1;cursor:pointer;padding:4px 8px;`;
const ErrorText=styled.div`padding:24px 0;color:#d45769;`;

const StationLoader=()=> <div>{Array.from({length:6}).map((_,i)=><ContentLoader key={i} speed={1.6} width="100%" height={65} viewBox="0 0 700 65" backgroundColor="#26222d" foregroundColor="#3a3345"><rect x="0" y="10" rx="6" ry="6" width="44" height="44"/><rect x="60" y="15" rx="4" ry="4" width="52%" height="13"/><rect x="60" y="38" rx="3" ry="3" width="34%" height="9"/></ContentLoader>)}</div>;

async function gql(query:string, variables:any={}) {
  return fetcher<any, any>(query, variables)();
}
const fields=`id name streamUrl source genre country logo bitrate`;

export default function RadioPage(){
  const [searchParams,setSearchParams]=useSearchParams();
  const [tab,setTab]=useState<'search'|'saved'>('search'); const [stations,setStations]=useState<Station[]>([]); const [saved,setSaved]=useState<Station[]>([]); const [search,setSearch]=useState(false); const [q,setQ]=useState('');
  const [category,setCategory]=useState<{label:string;term:string}>(); const [loading,setLoading]=useState(false); const [error,setError]=useState(''); const requestId=useRef(0);
  const loadSaved=async()=>{const d=await gql(`query { savedRadios { ${fields} } }`);setSaved(d.savedRadios||[])};
  useEffect(()=>{loadSaved()},[]);
  useEffect(()=>{if(searchParams.get('search')==='1')setSearch(true)},[searchParams]);
  const discover=async(category?:string,query?:string)=>{const id=++requestId.current;setLoading(true);setError('');try{const d=await gql(`query($query:String,$category:String){ radios(query:$query,category:$category){${fields}}}`,{query,category});if(id===requestId.current)setStations(d.radios||[])}catch(e){if(id===requestId.current){setStations([]);setError(e instanceof Error?e.message:'Unable to load radio stations')}}finally{if(id===requestId.current)setLoading(false)}};
  const openCategory=(item:{label:string;term:string})=>{setCategory(item);setStations([]);discover(item.term)};
  const play=(s:Station)=>gql(`mutation($station:RadioStationInput!){playRadio(station:$station)}`,{station:s});
  const toggle=async(s:Station)=>{const exists=saved.some(x=>x.id===s.id);await gql(exists?`mutation($id:String!){removeSavedRadio(id:$id)}`:`mutation($station:RadioStationInput!){saveRadio(station:$station)}`,exists?{id:s.id}:{station:s});await loadSaved()};
  const stationRows=(items:Station[])=>items.map(s=><Row key={s.id}>{s.logo?<Logo src={s.logo}/>:<div>◉</div>}<Info><Name title={s.name}>{s.name}</Name><Meta>{[s.genre,s.country,s.source].filter(Boolean).join(' · ')}</Meta></Info><Action onClick={()=>toggle(s)}>{saved.some(x=>x.id===s.id)?'♥':'♡'}</Action><Action onClick={()=>play(s)}>▶</Action></Row>);
  const closeSearch=()=>{setSearch(false);setTab('search');if(searchParams.has('search'))setSearchParams({}, {replace:true})};
  const tabOverrides={Tab:{style:{backgroundColor:"transparent",borderRadius:0}},TabPanel:{style:{paddingLeft:0,paddingRight:0}}};
  return <Shell><Sidebar active="radio"/><Main><ControlBar/><Body><h1>Internet Radio</h1>{category?<><CategoryHeader><BackButton onClick={()=>{setCategory(undefined);setStations([])}}>‹</BackButton><h2>{category.label}</h2></CategoryHeader>{loading?<StationLoader/>:error?<ErrorText>{error}</ErrorText>:stationRows(stations)}</>:<Tabs activeKey={tab} onChange={({activeKey})=>setTab(activeKey as 'search'|'saved')} overrides={{TabList:{style:{marginLeft:0,marginRight:0}},TabBorder:{style:{marginLeft:0,marginRight:0}},TabHighlight:{style:{height:"2px",backgroundColor:"#ab28fc"}}}}><Tab key="search" title="Search" overrides={tabOverrides}><SearchLauncher onClick={()=>setSearch(true)}><SearchIcon size={21}/><span>Search radio stations…</span></SearchLauncher><Grid>{categories.map(c=><Category key={c.label} onClick={()=>openCategory(c)}><CategoryIcon color={c.color}><c.Icon size={19}/></CategoryIcon><span>{c.label}</span></Category>)}</Grid></Tab><Tab key="saved" title="Bookmarked" overrides={tabOverrides}>{stationRows(saved)}</Tab></Tabs>}</Body></Main>{search&&<Modal onClick={closeSearch}><Panel onClick={e=>e.stopPropagation()}><SearchField><SearchIcon size={21}/><Input autoFocus placeholder="Search radio stations…" value={q} onChange={e=>{const value=e.target.value;setQ(value);if(value.trim())discover(undefined,value);else{requestId.current++;setStations([]);setLoading(false)}}}/></SearchField>{loading?<StationLoader/>:error?<ErrorText>{error}</ErrorText>:stations.map(s=><Row key={s.id}><Info><Name title={s.name}>{s.name}</Name><Meta>{s.source}</Meta></Info><Action onClick={()=>toggle(s)}>♡</Action><Action onClick={()=>{play(s);closeSearch()}}>▶</Action></Row>)}</Panel></Modal>}</Shell>
}
