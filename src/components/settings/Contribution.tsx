import React from "react";
import styles from "../../styles/App.module.css";
import { Bug, Lightbulb, FileText, GitPullRequest, Heart, ExternalLink } from "lucide-react";
import { GithubIcon } from "../GithubIcon";

interface ContributionProps {
  appName: string;
}

export const Contribution: React.FC<ContributionProps> = ({
  appName
}) => {
  return (
    <section className={styles.settingsGroup}>
      <div className={styles.settingsHeader}>
        <h2>Contribution</h2>
        <p>{appName} is open source. Help us shape the future of privacy.</p>
      </div>
      
      <div className={styles.contributionGrid}>
        <a href="https://github.com/RameshXT/windows-applock/issues/new?template=bug_report.md" target="_blank" rel="noopener noreferrer" className={styles.contribCard}>
          <div className={styles.contribIcon}><Bug size={20} /></div>
          <div className={styles.contribContent}>
            <span className={styles.contribTitle}>Report a Bug</span>
            <span className={styles.contribDesc}>Help us identify and squash technical issues.</span>
          </div>
          <ExternalLink size={14} style={{ position: 'absolute', bottom: '1.25rem', right: '1.25rem', opacity: 0.2 }} />
        </a>

        <a href="https://github.com/RameshXT/windows-applock/issues/new?template=feature_request.md" target="_blank" rel="noopener noreferrer" className={styles.contribCard}>
          <div className={styles.contribBadge}>Popular</div>
          <div className={styles.contribIcon}><Lightbulb size={20} /></div>
          <div className={styles.contribContent}>
            <span className={styles.contribTitle}>Request Feature</span>
            <span className={styles.contribDesc}>Suggest new ideas to make AppLock better.</span>
          </div>
          <ExternalLink size={14} style={{ position: 'absolute', bottom: '1.25rem', right: '1.25rem', opacity: 0.2 }} />
        </a>

        <a href="https://github.com/RameshXT/windows-applock/blob/main/CONTRIBUTING.md" target="_blank" rel="noopener noreferrer" className={styles.contribCard}>
          <div className={styles.contribIcon}><FileText size={20} /></div>
          <div className={styles.contribContent}>
            <span className={styles.contribTitle}>Documentation</span>
            <span className={styles.contribDesc}>Improve guides or clarify instructions.</span>
          </div>
          <ExternalLink size={14} style={{ position: 'absolute', bottom: '1.25rem', right: '1.25rem', opacity: 0.2 }} />
        </a>

        <a href="https://github.com/RameshXT/windows-applock/pulls" target="_blank" rel="noopener noreferrer" className={styles.contribCard}>
          <div className={styles.contribIcon}><GitPullRequest size={20} /></div>
          <div className={styles.contribContent}>
            <span className={styles.contribTitle}>Pull Requests</span>
            <span className={styles.contribDesc}>Submit code directly to the repository.</span>
          </div>
          <ExternalLink size={14} style={{ position: 'absolute', bottom: '1.25rem', right: '1.25rem', opacity: 0.2 }} />
        </a>

        <a href="https://github.com/RameshXT/windows-applock/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22" target="_blank" rel="noopener noreferrer" className={styles.contribCard}>
          <div className={styles.contribBadge}>Beginner</div>
          <div className={styles.contribIcon}><Heart size={20} /></div>
          <div className={styles.contribContent}>
            <span className={styles.contribTitle}>First Contribution</span>
            <span className={styles.contribDesc}>Find easy tasks curated for newcomers.</span>
          </div>
          <ExternalLink size={14} style={{ position: 'absolute', bottom: '1.25rem', right: '1.25rem', opacity: 0.2 }} />
        </a>

        <a href="https://github.com/RameshXT/windows-applock" target="_blank" rel="noopener noreferrer" className={styles.contribCard}>
          <div className={styles.contribIcon}><GithubIcon size={20} /></div>
          <div className={styles.contribContent}>
            <span className={styles.contribTitle}>Source Code</span>
            <span className={styles.contribDesc}>Browse the official project repository.</span>
          </div>
          <ExternalLink size={14} style={{ position: 'absolute', bottom: '1.25rem', right: '1.25rem', opacity: 0.2 }} />
        </a>
      </div>

      <div className={styles.workflowSection}>
        <h3 className={styles.workflowTitle}>How it works</h3>
        <div className={styles.workflowSteps}>
          <div className={styles.workflowConnector} />
          
          <div className={styles.workflowStep}>
            <div className={styles.stepNumber}>1</div>
            <span className={styles.stepLabel}>Fork</span>
            <span className={styles.stepDesc}>Copy repo</span>
          </div>

          <div className={styles.workflowStep}>
            <div className={styles.stepNumber}>2</div>
            <span className={styles.stepLabel}>Branch</span>
            <span className={styles.stepDesc}>Create feature</span>
          </div>

          <div className={styles.workflowStep}>
            <div className={styles.stepNumber}>3</div>
            <span className={styles.stepLabel}>Push</span>
            <span className={styles.stepDesc}>Commit code</span>
          </div>

          <div className={styles.workflowStep}>
            <div className={styles.stepNumber}>4</div>
            <span className={styles.stepLabel}>Merge</span>
            <span className={styles.stepDesc}>Pull Request</span>
          </div>
        </div>
      </div>
    </section>
  );
};
