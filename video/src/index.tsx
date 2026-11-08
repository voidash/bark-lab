import { Composition, registerRoot } from "remotion";
import { BarkBand, type BandProps } from "./BarkBand.tsx";

const defaults: BandProps = {
  title: "Who Let The Dogs Out",
  durationSeconds: 22,
  audio: null,
  repo: "github.com/voidash/bark-lab",
  levels: [], spectrum: [], events: [],
};

function Root() {
  return <Composition id="BarkBand" component={BarkBand} width={1920} height={1080} fps={30}
    durationInFrames={660} defaultProps={defaults}
    calculateMetadata={({props})=>{
      if (!Number.isFinite(props.durationSeconds) || props.durationSeconds<5 || props.durationSeconds>120) throw new Error("Invalid video duration");
      return {durationInFrames:Math.ceil(props.durationSeconds*30)};
    }}/>
}
registerRoot(Root);
