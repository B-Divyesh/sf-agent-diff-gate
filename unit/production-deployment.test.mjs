import assert from 'node:assert/strict';
import test from 'node:test';

import {
  DATABASE_URL,
  DEPLOYMENT_CONFIG_VERSION,
  assertProductionContract,
  productionContractErrors,
  renderProductionTemplate,
} from '../deploy/production-contract.mjs';
import { assertRateLimitResults } from '../deploy/live-rate-limit.mjs';

const runtime = {
  PUBLIC_BASE_URL: 'https://agent-diff-gate.sociobot.in',
  ENTRA_AUTHORITY: 'https://sociobotcustomers.ciamlogin.com/tenant',
  ENTRA_TENANT_ID: 'tenant',
  ENTRA_CLIENT_ID: 'client',
  ENTRA_TEAM_CLAIM: 'oid',
};
const image = 'sociobotregistry.azurecr.io/sf-agent-diff-gate:repair-sha';
const storageName = 'agent-diff-gate-data-v4';
const verifier15CandidateImage = 'sociobotregistry.azurecr.io/sf-agent-diff-gate:43c2f38a2e95';
const verifier16CandidateImage = 'sociobotregistry.azurecr.io/sf-agent-diff-gate:88c392a7825d';
const verifier17CandidateImage = 'sociobotregistry.azurecr.io/sf-agent-diff-gate:cfdd80845d42';
const verifier19CandidateImage = 'sociobotregistry.azurecr.io/sf-agent-diff-gate:9df61fc1e555';

function factoryStatelessApp() {
  return {
    properties: {
      configuration: { activeRevisionsMode: 'Single' },
      template: {
        containers: [
          {
            name: 'app',
            image: 'sociobotregistry.azurecr.io/sf-agent-diff-gate:old-sha',
            resources: { cpu: 0.5, memory: '1Gi' },
            env: [
              { name: 'PORT', value: '8080' },
              { name: 'UNCHANGED_SECRET', secretRef: 'github-key' },
            ],
          },
        ],
        scale: { minReplicas: 1, maxReplicas: 3, cooldownPeriod: 300 },
        volumes: null,
      },
    },
  };
}

function verifier14LiveApp() {
  const app = factoryStatelessApp();
  app.properties.template.containers[0].image = image;
  app.properties.template.containers[0].env = [{ name: 'PORT', value: '8080' }];
  return app;
}

function verifier15LiveApp() {
  const app = factoryStatelessApp();
  app.properties.template.containers[0].image = verifier15CandidateImage;
  app.properties.template.containers[0].env = [{ name: 'PORT', value: '8080' }];
  return app;
}

function verifier16LiveApp() {
  const app = factoryStatelessApp();
  app.properties.latestRevisionName = 'sf-agent-diff-gate--0000061';
  app.properties.template.containers[0].image = verifier16CandidateImage;
  app.properties.template.containers[0].env = [{ name: 'PORT', value: '8080' }];
  app.properties.template.scale = {
    cooldownPeriod: 300,
    maxReplicas: 3,
    minReplicas: 1,
    pollingInterval: 30,
    rules: null,
  };
  return app;
}

function verifier17LiveApp() {
  // Exact configuration captured by verification 17: the candidate image
  // ran with one-to-three replicas, only PORT, and no durable volume/mount.
  const app = factoryStatelessApp();
  app.properties.latestRevisionName = 'sf-agent-diff-gate--0000067';
  app.properties.template.containers[0].image = verifier17CandidateImage;
  app.properties.template.containers[0].env = [{ name: 'PORT', value: '8080' }];
  app.properties.template.scale = {
    cooldownPeriod: 300,
    maxReplicas: 3,
    minReplicas: 1,
    pollingInterval: 30,
    rules: null,
  };
  return app;
}

function verifier19LiveApp() {
  // Exact read-only control-plane capture from verification 19. Although
  // active revision mode was Single, its template still allowed three
  // replicas and supplied only PORT, so all durable SQLite and identity
  // invariants were absent. That is enough to fail-close real work and to
  // make an in-process limiter nondeterministically multiply per replica.
  const app = factoryStatelessApp();
  app.properties.latestRevisionName = 'sf-agent-diff-gate--0000073';
  app.properties.template.containers[0].image = verifier19CandidateImage;
  app.properties.template.containers[0].env = [{ name: 'PORT', value: '8080' }];
  app.properties.template.scale = {
    cooldownPeriod: 300,
    maxReplicas: 3,
    minReplicas: 1,
    pollingInterval: 30,
    rules: null,
  };
  return app;
}

test('regression: verifier 14 generic three-replica deployment is rejected in full', () => {
  // This is the failing Azure control-plane shape recorded by verification 14:
  // a candidate image but only PORT, three possible replicas, and no volume.
  const failingLiveConfiguration = verifier14LiveApp();
  const errors = productionContractErrors(failingLiveConfiguration, { image, storageName, runtime });
  assert.deepEqual(errors, [
    'SQLite requires exactly one replica',
    'Azure Files volume data must use agent-diff-gate-data-v4',
    'Azure Files volume data must be mounted at /data',
    'DATABASE_URL must match the production contract',
    'PUBLIC_BASE_URL must match the production contract',
    'ENTRA_AUTHORITY must match the production contract',
    'ENTRA_TENANT_ID must match the production contract',
    'ENTRA_CLIENT_ID must match the production contract',
    'ENTRA_TEAM_CLAIM must match the production contract',
    'DEPLOYMENT_CONFIG_VERSION must match the production contract',
  ]);
  assert.throws(
    () => assertProductionContract(failingLiveConfiguration, { image, storageName, runtime }),
    /Unsafe production configuration/,
  );
});

test('regression: verifier 15 candidate image cannot pass with the generic stateless topology', () => {
  // This is the precise control-plane shape reported in verification 15. The
  // candidate image itself is not evidence of a safe release: SQLite also
  // needs its single-replica limit, Azure Files mount, and complete runtime
  // contract.
  const errors = productionContractErrors(verifier15LiveApp(), {
    image: verifier15CandidateImage,
    storageName,
    runtime,
  });

  assert.deepEqual(errors, [
    'SQLite requires exactly one replica',
    'Azure Files volume data must use agent-diff-gate-data-v4',
    'Azure Files volume data must be mounted at /data',
    'DATABASE_URL must match the production contract',
    'PUBLIC_BASE_URL must match the production contract',
    'ENTRA_AUTHORITY must match the production contract',
    'ENTRA_TENANT_ID must match the production contract',
    'ENTRA_CLIENT_ID must match the production contract',
    'ENTRA_TEAM_CLAIM must match the production contract',
    'DEPLOYMENT_CONFIG_VERSION must match the production contract',
  ]);
  assert.throws(
    () =>
      assertProductionContract(verifier15LiveApp(), {
        image: verifier15CandidateImage,
        storageName,
        runtime,
      }),
    /Unsafe production configuration/,
  );
});

test('regression: verifier 16 live revision is rejected and repaired as one stateful replica', () => {
  // Exact control-plane fixture from verification 16: the right image was not
  // enough because the generic release recreated a PORT-only three-replica
  // template with three independent SQLite databases and limiters.
  const failingLiveConfiguration = verifier16LiveApp();
  const errors = productionContractErrors(failingLiveConfiguration, {
    image: verifier16CandidateImage,
    storageName,
    runtime,
  });

  assert.deepEqual(errors, [
    'SQLite requires exactly one replica',
    'Azure Files volume data must use agent-diff-gate-data-v4',
    'Azure Files volume data must be mounted at /data',
    'DATABASE_URL must match the production contract',
    'PUBLIC_BASE_URL must match the production contract',
    'ENTRA_AUTHORITY must match the production contract',
    'ENTRA_TENANT_ID must match the production contract',
    'ENTRA_CLIENT_ID must match the production contract',
    'ENTRA_TEAM_CLAIM must match the production contract',
    'DEPLOYMENT_CONFIG_VERSION must match the production contract',
  ]);

  const repaired = structuredClone(failingLiveConfiguration);
  repaired.properties.template = renderProductionTemplate(failingLiveConfiguration, {
    image: verifier16CandidateImage,
    storageName,
    runtime,
  });
  assert.doesNotThrow(() =>
    assertProductionContract(repaired, { image: verifier16CandidateImage, storageName, runtime }),
  );
});

test('regression: verifier 17 exact candidate topology is rejected and rendered safe', () => {
  const failingLiveConfiguration = verifier17LiveApp();
  const errors = productionContractErrors(failingLiveConfiguration, {
    image: verifier17CandidateImage,
    storageName,
    runtime,
  });

  assert.deepEqual(errors, [
    'SQLite requires exactly one replica',
    'Azure Files volume data must use agent-diff-gate-data-v4',
    'Azure Files volume data must be mounted at /data',
    'DATABASE_URL must match the production contract',
    'PUBLIC_BASE_URL must match the production contract',
    'ENTRA_AUTHORITY must match the production contract',
    'ENTRA_TENANT_ID must match the production contract',
    'ENTRA_CLIENT_ID must match the production contract',
    'ENTRA_TEAM_CLAIM must match the production contract',
    'DEPLOYMENT_CONFIG_VERSION must match the production contract',
  ]);
  assert.throws(
    () => assertProductionContract(failingLiveConfiguration, {
      image: verifier17CandidateImage,
      storageName,
      runtime,
    }),
    /Unsafe production configuration/,
  );

  const repaired = {
    properties: {
      configuration: { activeRevisionsMode: 'Single' },
      template: renderProductionTemplate(failingLiveConfiguration, {
        image: verifier17CandidateImage,
        storageName,
        runtime,
      }),
    },
  };
  assert.doesNotThrow(() =>
    assertProductionContract(repaired, { image: verifier17CandidateImage, storageName, runtime }),
  );
});

test('regression: verifier 19 exact PORT-only topology and 80-request allowance are rejected', () => {
  const failingLiveConfiguration = verifier19LiveApp();
  const errors = productionContractErrors(failingLiveConfiguration, {
    image: verifier19CandidateImage,
    storageName,
    runtime,
  });

  assert.deepEqual(errors, [
    'SQLite requires exactly one replica',
    'Azure Files volume data must use agent-diff-gate-data-v4',
    'Azure Files volume data must be mounted at /data',
    'DATABASE_URL must match the production contract',
    'PUBLIC_BASE_URL must match the production contract',
    'ENTRA_AUTHORITY must match the production contract',
    'ENTRA_TENANT_ID must match the production contract',
    'ENTRA_CLIENT_ID must match the production contract',
    'ENTRA_TEAM_CLAIM must match the production contract',
    'DEPLOYMENT_CONFIG_VERSION must match the production contract',
  ]);
  assert.throws(
    () =>
      assertProductionContract(failingLiveConfiguration, {
        image: verifier19CandidateImage,
        storageName,
        runtime,
      }),
    /Unsafe production configuration/,
  );

  const twoReplicaRateResult = [
    ...Array.from({ length: 80 }, () => ({ status: 200, retryAfter: null })),
    ...Array.from({ length: 20 }, () => ({ status: 429, retryAfter: '1' })),
  ];
  assert.throws(
    () => assertRateLimitResults(twoReplicaRateResult),
    /accepted 80 requests; expected exactly 40 from one client/,
  );

  const repaired = {
    properties: {
      configuration: { activeRevisionsMode: 'Single' },
      template: renderProductionTemplate(failingLiveConfiguration, {
        image: verifier19CandidateImage,
        storageName,
        runtime,
      }),
    },
  };
  assert.doesNotThrow(() =>
    assertProductionContract(repaired, { image: verifier19CandidateImage, storageName, runtime }),
  );
});

test('regression: verifier 16 multiplied rate allowance fails the live probe', () => {
  const expected = [
    ...Array.from({ length: 40 }, () => ({ status: 200, retryAfter: null })),
    ...Array.from({ length: 60 }, () => ({ status: 429, retryAfter: '1' })),
  ];
  assert.deepEqual(assertRateLimitResults(expected), { accepted: 40, rejected: 60, retryAfter: '1' });

  const threeReplicaResult = [
    ...Array.from({ length: 120 }, () => ({ status: 200, retryAfter: null })),
    ...Array.from({ length: 30 }, () => ({ status: 429, retryAfter: '1' })),
  ];
  assert.throws(
    () => assertRateLimitResults(threeReplicaResult, { total: 150 }),
    /accepted 120 requests; expected exactly 40/,
  );

  const missingRetryAfter = structuredClone(expected);
  missingRetryAfter.at(-1).retryAfter = null;
  assert.throws(() => assertRateLimitResults(missingRetryAfter), /did not include Retry-After: 1/);
});

test('regression: a duplicate managed variable or data mount cannot mask an unsafe contract', () => {
  const template = renderProductionTemplate(factoryStatelessApp(), { image, storageName, runtime });
  template.containers[0].env.push({ name: 'DATABASE_URL', value: 'sqlite:/tmp/split.db?mode=rwc' });
  template.containers[0].volumeMounts.push({ volumeName: 'data', mountPath: '/tmp' });

  const errors = productionContractErrors(
    { properties: { configuration: { activeRevisionsMode: 'Single' }, template } },
    { image, storageName, runtime },
  );
  assert.deepEqual(errors, [
    'Azure Files volume data must be mounted at /data',
    'DATABASE_URL must match the production contract',
  ]);
});

test('regression: one render atomically installs the image and durable SQLite contract', () => {
  const original = factoryStatelessApp();
  const template = renderProductionTemplate(original, { image, storageName, runtime });
  const repaired = {
    properties: {
      configuration: { activeRevisionsMode: 'Single' },
      template,
    },
  };

  assert.doesNotThrow(() => assertProductionContract(repaired, { image, storageName, runtime }));
  assert.deepEqual(template.scale, { minReplicas: 1, maxReplicas: 1 });
  assert.deepEqual(template.volumes, [{ name: 'data', storageType: 'AzureFile', storageName }]);
  assert.deepEqual(template.containers[0].volumeMounts, [{ volumeName: 'data', mountPath: '/data' }]);
  assert.equal(template.containers[0].image, image);
  assert(template.containers[0].env.some(entry => entry.name === 'DATABASE_URL' && entry.value === DATABASE_URL));
  assert(
    template.containers[0].env.some(
      entry => entry.name === 'DEPLOYMENT_CONFIG_VERSION' && entry.value === DEPLOYMENT_CONFIG_VERSION,
    ),
  );
  assert(template.containers[0].env.some(entry => entry.name === 'UNCHANGED_SECRET' && entry.secretRef === 'github-key'));
  assert.deepEqual(original, factoryStatelessApp(), 'rendering must not mutate the Azure response');
});

test('regression: scale, storage, mount, database path, and image are all mandatory', () => {
  const base = factoryStatelessApp();
  const safeTemplate = renderProductionTemplate(base, { image, storageName, runtime });
  const mutations = [
    template => {
      template.scale.maxReplicas = 3;
    },
    template => {
      template.volumes = null;
    },
    template => {
      template.containers[0].volumeMounts = [];
    },
    template => {
      template.containers[0].env.find(entry => entry.name === 'DATABASE_URL').value = 'sqlite:/data/other.db';
    },
    template => {
      template.containers[0].image = 'sociobotregistry.azurecr.io/sf-agent-diff-gate:wrong';
    },
  ];

  for (const mutate of mutations) {
    const template = structuredClone(safeTemplate);
    mutate(template);
    assert.throws(
      () =>
        assertProductionContract(
          { properties: { configuration: { activeRevisionsMode: 'Single' }, template } },
          { image, storageName, runtime },
        ),
      /Unsafe production configuration/,
    );
  }
});
