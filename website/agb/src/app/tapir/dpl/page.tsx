import { ContentBlock } from "@/components/contentBlock";
import { Redirect } from "@/components/redirect";
import { Metadata } from "next";

export const metadata: Metadata = {
  title: "Dungeon Puzzler Redirect",
};

const REDIRECT_TO = "/showcase/the-dungeon-puzzlers-lament";

export default function DplRedirectPage() {
  return (
    <>
      <Redirect url={REDIRECT_TO} />
      <ContentBlock>
        <h1>This page is a redirect to the Dungeon Puzzler</h1>
      </ContentBlock>
      <ContentBlock>
        <p>
          You should be redirected automatically.{" "}
          <a href={REDIRECT_TO}>
            If you were not redirected automatically click here.
          </a>
        </p>
      </ContentBlock>
    </>
  );
}
